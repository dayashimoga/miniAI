import 'package:flutter/foundation.dart';
import '../models/system_contracts.dart';

/// Reactive controller managing Local-Only Privacy Mode, CPU-first execution,
/// explicit cloud consent tokens, and independent modality adapter statuses.
class PlatformController extends ChangeNotifier {
  PlatformController({
    HardwareProfile? initialHardware,
  }) : _hardware = initialHardware ??
            const HardwareProfile(
              osFamily: 'windows',
              arch: 'x86_64',
              logicalCpuCores: 8,
              totalMemoryBytes: 16 * 1024 * 1024 * 1024,
              availableMemoryBytes: 8 * 1024 * 1024 * 1024,
              detectedBackends: [AcceleratorBackend.cpu],
            ) {
    _rebuildAdapterHealth();
  }

  final HardwareProfile _hardware;
  bool _localOnlyMode = true;
  bool _forceCpuOnly = false;
  CloudConsentToken? _activeConsent;
  Modality _selectedModality = Modality.llm;
  late List<AdapterHealth> _adapterHealthList;
  String _lastAuditMessage =
      'Local-Only Privacy Mode initialized (outbound cloud blocked).';

  HardwareProfile get hardware => _hardware;
  bool get localOnlyMode => _localOnlyMode;
  bool get forceCpuOnly => _forceCpuOnly;
  CloudConsentToken? get activeConsent => _activeConsent;
  Modality get selectedModality => _selectedModality;
  List<AdapterHealth> get adapterHealthList =>
      List.unmodifiable(_adapterHealthList);
  String get lastAuditMessage => _lastAuditMessage;

  AcceleratorBackend get activeExecutionBackend =>
      _hardware.selectExecutionBackend(forceCpu: _forceCpuOnly);

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
    notifyListeners();
  }

  void setForceCpuOnly(bool forceCpu) {
    _forceCpuOnly = forceCpu;
    _rebuildAdapterHealth();
    _lastAuditMessage = forceCpu
        ? 'Forced CPU-Only execution enabled across all modality adapters.'
        : 'Hardware auto-selection enabled (CPU baseline fallback active).';
    notifyListeners();
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
    notifyListeners();
    return true;
  }

  void revokeCloudConsent() {
    _activeConsent = null;
    _lastAuditMessage = 'CloudConsentToken revoked by user.';
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

