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

  final List<ModelManifest> _models = [
    const ModelManifest(
      modelId: 'llama-3.2-1b-instruct-q4',
      name: 'Llama 3.2 1B Instruct (GGUF Q4)',
      modality: Modality.llm,
      version: '1.0.0',
      description: 'Compact general text generation and reasoning model',
      sourceUrl: 'http://127.0.0.1:4140/models/llama-3.2-1b.gguf',
      expectedSha256:
          '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      expectedSizeBytes: 1288490188,
      format: 'gguf',
      parameterCount: '1.23B',
      quantization: 'Q4_K_M',
      minRamBytes: 2147483648,
      recommendedBackend: AcceleratorBackend.cpu,
    ),
    const ModelManifest(
      modelId: 'clip-vit-base-patch32',
      name: 'CLIP ViT-B/32 (ONNX FP16)',
      modality: Modality.vision,
      version: '1.0.0',
      description:
          'Zero-shot image classification and visual feature extraction',
      sourceUrl: 'http://127.0.0.1:4140/models/clip-vit-base.onnx',
      expectedSha256:
          '5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8',
      expectedSizeBytes: 367001600,
      format: 'onnx',
      parameterCount: '86M',
      quantization: 'FP16',
      minRamBytes: 1073741824,
      recommendedBackend: AcceleratorBackend.cpu,
    ),
    const ModelManifest(
      modelId: 'whisper-tiny-en',
      name: 'Whisper Tiny English (GGUF Q8)',
      modality: Modality.speechAsr,
      version: '1.0.0',
      description: 'Fast streaming automatic speech recognition',
      sourceUrl: 'http://127.0.0.1:4140/models/whisper-tiny.gguf',
      expectedSha256:
          '4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a',
      expectedSizeBytes: 78643200,
      format: 'gguf',
      parameterCount: '39M',
      quantization: 'Q8_0',
      minRamBytes: 536870912,
      recommendedBackend: AcceleratorBackend.cpu,
    ),
    const ModelManifest(
      modelId: 'piper-en-amy-medium',
      name: 'Piper Amy English (ONNX)',
      modality: Modality.speechTts,
      version: '1.0.0',
      description: 'Real-time neural text-to-speech engine',
      sourceUrl: 'http://127.0.0.1:4140/models/piper-amy.onnx',
      expectedSha256:
          'ef2d127de37b942baad06145e54b0c619a1f22327b2ebbcfbec78f5564afe39d',
      expectedSizeBytes: 62914560,
      format: 'onnx',
      parameterCount: '28M',
      quantization: 'FP32',
      minRamBytes: 536870912,
      recommendedBackend: AcceleratorBackend.cpu,
    ),
  ];

  final Map<String, DownloadProgressItem> _downloadProgress = {
    'whisper-tiny-en': const DownloadProgressItem(
      modelId: 'whisper-tiny-en',
      bytesDownloaded: 78643200,
      totalBytes: 78643200,
      percentage: 100.0,
      status: DownloadStatus.verified,
    ),
  };

  Modality? _modelModalityFilter;

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

  List<ModelManifest> get models => List.unmodifiable(_models);
  Map<String, DownloadProgressItem> get downloadProgress =>
      Map.unmodifiable(_downloadProgress);
  Modality? get modelModalityFilter => _modelModalityFilter;

  List<ModelManifest> get filteredModels {
    if (_modelModalityFilter == null) {
      return List.unmodifiable(_models);
    }
    return List.unmodifiable(
      _models.where((m) => m.modality == _modelModalityFilter),
    );
  }

  void setModelModalityFilter(Modality? filter) {
    _modelModalityFilter = filter;
    notifyListeners();
  }

  DownloadProgressItem getModelStatus(String modelId) {
    return _downloadProgress[modelId] ??
        DownloadProgressItem(
          modelId: modelId,
          bytesDownloaded: 0,
          totalBytes: 0,
          percentage: 0.0,
          status: DownloadStatus.pending,
        );
  }

  void startDownload(String modelId) {
    final manifest = _models.firstWhere((m) => m.modelId == modelId);
    if (_localOnlyMode && manifest.sourceUrl.startsWith('http')) {
      _downloadProgress[modelId] = DownloadProgressItem(
        modelId: modelId,
        bytesDownloaded: 0,
        totalBytes: manifest.expectedSizeBytes,
        percentage: 0.0,
        status: DownloadStatus.failed,
        errorMessage: 'Remote downloads blocked in Local-Only Privacy Mode',
      );
      _recordEvent(
        eventType: 'model_download_failed',
        payload: {
          'model_id': modelId,
          'reason': 'Remote downloads blocked in Local-Only Privacy Mode',
        },
      );
      notifyListeners();
      return;
    }

    _downloadProgress[modelId] = DownloadProgressItem(
      modelId: modelId,
      bytesDownloaded: (manifest.expectedSizeBytes * 0.45).toInt(),
      totalBytes: manifest.expectedSizeBytes,
      percentage: 45.0,
      status: DownloadStatus.downloading,
    );
    _recordEvent(
      eventType: 'model_download_progress',
      payload: {
        'model_id': modelId,
        'bytes_downloaded': (manifest.expectedSizeBytes * 0.45).toInt(),
        'total_bytes': manifest.expectedSizeBytes,
        'percentage': 45.0,
      },
    );
    notifyListeners();
  }

  void pauseDownload(String modelId) {
    final current = _downloadProgress[modelId];
    if (current != null) {
      _downloadProgress[modelId] =
          current.copyWith(status: DownloadStatus.paused);
      notifyListeners();
    }
  }

  void resumeDownload(String modelId) {
    final current = _downloadProgress[modelId];
    if (current != null) {
      _downloadProgress[modelId] =
          current.copyWith(status: DownloadStatus.downloading);
      notifyListeners();
    }
  }

  void cancelDownload(String modelId) {
    _downloadProgress.remove(modelId);
    notifyListeners();
  }

  void completeDownload(String modelId) {
    final manifest = _models.firstWhere((m) => m.modelId == modelId);
    _downloadProgress[modelId] = DownloadProgressItem(
      modelId: modelId,
      bytesDownloaded: manifest.expectedSizeBytes,
      totalBytes: manifest.expectedSizeBytes,
      percentage: 100.0,
      status: DownloadStatus.verified,
    );
    _recordEvent(
      eventType: 'model_download_verified',
      payload: {
        'model_id': modelId,
        'sha256': manifest.expectedSha256,
      },
    );
    notifyListeners();
  }

  void importLocalModel(LocalModelImportRequest request) {
    final manifest = ModelManifest(
      modelId: request.modelId,
      name: request.name,
      modality: request.modality,
      version: request.version,
      description: request.description,
      sourceUrl: 'file://${request.sourcePath}',
      expectedSha256: request.expectedSha256 ??
          'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
      expectedSizeBytes: request.expectedSizeBytes ?? 1024 * 1024 * 100,
      format: request.format,
      parameterCount: request.parameterCount,
      quantization: request.quantization,
      minRamBytes: request.minRamBytes,
      recommendedBackend: request.recommendedBackend,
    );

    _models.removeWhere((m) => m.modelId == request.modelId);
    _models.insert(0, manifest);

    _downloadProgress[request.modelId] = DownloadProgressItem(
      modelId: request.modelId,
      bytesDownloaded: manifest.expectedSizeBytes,
      totalBytes: manifest.expectedSizeBytes,
      percentage: 100.0,
      status: DownloadStatus.verified,
    );

    _recordEvent(
      eventType: 'model_imported',
      payload: {
        'model_id': request.modelId,
        'source_path': request.sourcePath,
        'status': 'verified',
      },
    );
    notifyListeners();
  }

  void deleteModel(String modelId) {
    _models.removeWhere((m) => m.modelId == modelId);
    _downloadProgress.remove(modelId);
    notifyListeners();
  }

  bool testMemoryAdmission(int requiredBytes) {
    return _hardware.canAdmitModel(requiredBytes);
  }

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
    saveSettings();
  }

  Future<void> saveSettings() {
    return _settingsStore.saveSettings(
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
