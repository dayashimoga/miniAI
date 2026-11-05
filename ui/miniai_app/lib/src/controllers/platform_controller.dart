import 'package:flutter/foundation.dart';
import '../models/system_contracts.dart';
import '../services/ipc_client.dart';
import '../services/settings_store.dart';

/// Reactive controller managing Local-Only Privacy Mode, CPU-first execution,
/// explicit cloud consent tokens, navigation, SQLite-backed settings persistence,
/// and live event streams across all 7 modality adapters.
class PlatformController extends ChangeNotifier {
  PlatformController({
    HardwareProfile? initialHardware,
    SettingsStore? settingsStore,
    IpcClient? ipcClient,
  })  : _hardware = initialHardware ??
            const HardwareProfile(
              osFamily: 'windows',
              arch: 'x86_64',
              logicalCpuCores: 8,
              totalMemoryBytes: 16 * 1024 * 1024 * 1024,
              availableMemoryBytes: 8 * 1024 * 1024 * 1024,
              detectedBackends: [AcceleratorBackend.cpu],
            ),
        _settingsStore = settingsStore ?? InMemorySettingsStore(),
        _ipcClient = ipcClient {
    _rebuildAdapterHealth();
    _recordEvent(
      eventType: 'privacy_policy_updated',
      payload: {
        'local_only_mode': _localOnlyMode,
        'message': _lastAuditMessage,
      },
    );
  }

  final HardwareProfile _hardware;
  final SettingsStore _settingsStore;
  IpcClient? _ipcClient;

  bool _localOnlyMode = true;
  bool _forceCpuOnly = false;
  String _activeWorkspaceId = 'default';
  String _ipcServerUrl = 'http://127.0.0.1:4140';
  String _sessionNonce = '';
  CloudConsentToken? _activeConsent;
  Modality _selectedModality = Modality.llm;
  late List<AdapterHealth> _adapterHealthList;
  String _lastAuditMessage =
      'Local-Only Privacy Mode initialized (outbound cloud blocked).';

  int _selectedNavigationIndex = 0;
  final List<EventEnvelope> _events = [];
  int _eventSequence = 0;
  String? _eventFilter;
  bool _ipcConnected = false;

  HardwareProfile get hardware => _hardware;
  SettingsStore get settingsStore => _settingsStore;
  IpcClient? get ipcClient => _ipcClient;
  bool get localOnlyMode => _localOnlyMode;
  bool get forceCpuOnly => _forceCpuOnly;
  String get activeWorkspaceId => _activeWorkspaceId;
  String get ipcServerUrl => _ipcServerUrl;
  String get sessionNonce => _sessionNonce;
  CloudConsentToken? get activeConsent => _activeConsent;
  Modality get selectedModality => _selectedModality;
  List<AdapterHealth> get adapterHealthList =>
      List.unmodifiable(_adapterHealthList);
  String get lastAuditMessage => _lastAuditMessage;
  int get selectedNavigationIndex => _selectedNavigationIndex;
  List<EventEnvelope> get events => List.unmodifiable(_events);
  String? get eventFilter => _eventFilter;
  bool get ipcConnected => _ipcConnected;

  List<EventEnvelope> get filteredEvents {
    if (_eventFilter == null || _eventFilter!.isEmpty || _eventFilter == 'all') {
      return List.unmodifiable(_events);
    }
    return List.unmodifiable(_events.where((e) => e.eventType == _eventFilter));
  }

  AcceleratorBackend get activeExecutionBackend =>
      _hardware.selectExecutionBackend(forceCpu: _forceCpuOnly);

  /// Load settings from persistence and synchronize state.
  Future<void> loadSettings() async {
    final settings = await _settingsStore.loadSettings();
    _localOnlyMode = settings.localOnlyMode;
    _forceCpuOnly = settings.forceCpuOnly;
    _activeWorkspaceId = settings.activeWorkspaceId;
    _ipcServerUrl = settings.ipcServerUrl;
    _sessionNonce = settings.sessionNonce;
    _rebuildAdapterHealth();
    _lastAuditMessage =
        'Loaded settings from persistent storage (workspace: $_activeWorkspaceId).';
    notifyListeners();
  }

  void setSelectedNavigationIndex(int index) {
    if (_selectedNavigationIndex != index) {
      _selectedNavigationIndex = index;
      notifyListeners();
    }
  }

  void selectModality(Modality modality) {
    if (_selectedModality != modality) {
      _selectedModality = modality;
      _lastAuditMessage = 'Selected modality workspace: ${modality.displayName}';
      notifyListeners();
    }
  }

  void setLocalOnlyMode(bool enabled) {
    _localOnlyMode = enabled;
    if (enabled) {
      _activeConsent = null;
      _lastAuditMessage =
          'Local-Only Privacy Mode enabled. Active cloud consent tokens revoked.';
    } else {
      _lastAuditMessage =
          'Local-Only Privacy Mode disabled. Explicit CloudConsentToken still required for remote execution.';
    }
    _recordEvent(
      eventType: 'privacy_policy_updated',
      payload: {
        'local_only_mode': enabled,
        'message': _lastAuditMessage,
      },
    );
    _saveSettings();
    notifyListeners();
  }

  void setForceCpuOnly(bool forceCpu) {
    _forceCpuOnly = forceCpu;
    _rebuildAdapterHealth();
    _lastAuditMessage = forceCpu
        ? 'Forced CPU-Only execution enabled across all modality adapters.'
        : 'Hardware auto-selection enabled (CPU baseline fallback active).';
    _recordEvent(
      eventType: 'hardware_changed',
      payload: {
        'force_cpu_only': forceCpu,
        'active_backend': activeExecutionBackend.wireValue,
        'message': _lastAuditMessage,
      },
    );
    _saveSettings();
    notifyListeners();
  }

  void setActiveWorkspace(String workspaceId) {
    final trimmed = workspaceId.trim();
    if (trimmed.isNotEmpty && _activeWorkspaceId != trimmed) {
      final previous = _activeWorkspaceId;
      _activeWorkspaceId = trimmed;
      _lastAuditMessage = 'Switched active workspace: $previous -> $trimmed';
      _recordEvent(
        eventType: 'workspace_switched',
        payload: {
          'previous_workspace_id': previous,
          'active_workspace_id': trimmed,
          'message': _lastAuditMessage,
        },
      );
      _saveSettings();
      notifyListeners();
    }
  }

  void setIpcConfig({required String url, required String nonce}) {
    _ipcServerUrl = url.trim();
    _sessionNonce = nonce.trim();
    _ipcClient = IpcClient(baseUrl: _ipcServerUrl, sessionNonce: _sessionNonce);
    _saveSettings();
    notifyListeners();
  }

  Future<bool> checkIpcHealth() async {
    _ipcClient ??= IpcClient(baseUrl: _ipcServerUrl, sessionNonce: _sessionNonce);
    final isHealthy = await _ipcClient!.checkHealth();
    _ipcConnected = isHealthy;
    _lastAuditMessage = isHealthy
        ? 'Connected to local IPC server at $_ipcServerUrl'
        : 'Local IPC server unavailable at $_ipcServerUrl';
    notifyListeners();
    return isHealthy;
  }

  bool grantCloudConsent(CloudConsentToken token) {
    if (_localOnlyMode) {
      _lastAuditMessage =
          'Blocked: Cannot grant CloudConsentToken while Local-Only Privacy Mode is active.';
      notifyListeners();
      return false;
    }
    if (!token.userAcknowledged ||
        token.tokenId.trim().isEmpty ||
        !token.targetEndpoint.startsWith('https://') ||
        token.expiresAtEpochSecs <= token.issuedAtEpochSecs) {
      _lastAuditMessage =
          'Rejected invalid CloudConsentToken: requires user acknowledgement, HTTPS endpoint, and valid expiry.';
      notifyListeners();
      return false;
    }
    _activeConsent = token;
    _lastAuditMessage =
        'Granted scoped CloudConsentToken (${token.tokenId}) for ${token.modality.displayName}.';
    _recordEvent(
      eventType: 'diagnostic_alert',
      payload: {
        'action': 'cloud_consent_granted',
        'token_id': token.tokenId,
        'modality': token.modality.wireValue,
      },
    );
    notifyListeners();
    return true;
  }

  void revokeCloudConsent() {
    _activeConsent = null;
    _lastAuditMessage = 'CloudConsentToken revoked by user.';
    _recordEvent(
      eventType: 'diagnostic_alert',
      payload: {
        'action': 'cloud_consent_revoked',
        'message': _lastAuditMessage,
      },
    );
    notifyListeners();
  }

  bool canExecuteCloudRequest({
    required Modality modality,
    required String endpoint,
    required int payloadBytes,
    required int currentEpochSecs,
  }) {
    if (_localOnlyMode || _activeConsent == null) {
      return false;
    }
    return _activeConsent!.isValidFor(
      requestedModality: modality,
      endpoint: endpoint,
      payloadBytes: payloadBytes,
      currentEpochSecs: currentEpochSecs,
    );
  }

  void addEvent(EventEnvelope event) {
    _events.insert(0, event);
    if (_events.length > 200) {
      _events.removeLast();
    }
    notifyListeners();
  }

  void clearEvents() {
    _events.clear();
    _lastAuditMessage = 'Diagnostic and system event history cleared.';
    notifyListeners();
  }

  void setEventFilter(String? filter) {
    _eventFilter = filter;
    notifyListeners();
  }

  void emitDiagnosticPing([String message = 'Diagnostic ping from UI shell']) {
    _lastAuditMessage = message;
    _recordEvent(
      eventType: 'diagnostic_alert',
      payload: {
        'message': message,
        'timestamp': DateTime.now().toUtc().toIso8601String(),
      },
    );
    notifyListeners();
  }

  void _saveSettings() {
    _settingsStore.saveSettings(
      AppSettings(
        localOnlyMode: _localOnlyMode,
        forceCpuOnly: _forceCpuOnly,
        activeWorkspaceId: _activeWorkspaceId,
        ipcServerUrl: _ipcServerUrl,
        sessionNonce: _sessionNonce,
      ),
    );
  }

  void _recordEvent({
    required String eventType,
    required Map<String, dynamic> payload,
  }) {
    _eventSequence += 1;
    final event = EventEnvelope(
      sequence: _eventSequence,
      timestampUtc: DateTime.now().toUtc().toIso8601String(),
      eventType: eventType,
      payload: payload,
    );
    _events.insert(0, event);
    if (_events.length > 200) {
      _events.removeLast();
    }
  }

  void _rebuildAdapterHealth() {
    final backend = activeExecutionBackend;
    _adapterHealthList = Modality.values
        .map(
          (modality) => AdapterHealth(
            adapterId: '${modality.wireValue}-cpu-adapter',
            modality: modality,
            ready: true,
            activeBackend: backend,
            statusMessage:
                '${modality.displayName} isolated adapter ready on ${backend.displayName}',
          ),
        )
        .toList();
  }
}
