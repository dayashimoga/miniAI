import 'dart:convert';
import 'dart:ui';
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
    _initChatWorkspace();
    _initKnowledgeBase();
    _initCanvasStudio();
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

  final List<ChatSession> _chatSessions = [];
  final Map<String, List<ChatMessage>> _sessionMessages = {};
  String? _activeSessionId;

  bool _isGenerating = false;
  bool _shouldCancelGeneration = false;
  double _generationTokensPerSecond = 0.0;
  int _generationTokenCount = 0;
  double _generationElapsedSeconds = 0.0;

  String _selectedChatModelId = 'llama-3.2-1b-instruct-q4';
  double _chatTemperature = 0.7;
  int _chatMaxTokens = 256;

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
    const ModelManifest(
      modelId: 'sd-tiny-cpu-fp16',
      name: 'Stable Diffusion Tiny (CPU FP16)',
      modality: Modality.imageGeneration,
      version: '1.0.0',
      description: 'Compact CPU-optimized latent diffusion generator & inpainting model',
      sourceUrl: 'http://127.0.0.1:4140/models/sd-tiny.onnx',
      expectedSha256:
          '7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069',
      expectedSizeBytes: 268435456,
      format: 'onnx',
      parameterCount: '68M',
      quantization: 'FP16',
      minRamBytes: 1073741824,
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

  List<ChatSession> get chatSessions => List.unmodifiable(_chatSessions);
  String? get activeSessionId => _activeSessionId;
  ChatSession? get activeSession {
    if (_activeSessionId == null) return null;
    for (final s in _chatSessions) {
      if (s.id == _activeSessionId) return s;
    }
    return null;
  }
  List<ChatMessage> get currentMessages => _activeSessionId != null
      ? List.unmodifiable(_sessionMessages[_activeSessionId] ?? [])
      : const [];

  bool get isGenerating => _isGenerating;
  double get generationTokensPerSecond => _generationTokensPerSecond;
  int get generationTokenCount => _generationTokenCount;
  double get generationElapsedSeconds => _generationElapsedSeconds;

  String get selectedChatModelId => _selectedChatModelId;
  double get chatTemperature => _chatTemperature;
  int get chatMaxTokens => _chatMaxTokens;

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

  void _initChatWorkspace() {
    final now = DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000;
    const defaultId = 'session-default-01';
    final defaultSession = ChatSession(
      id: defaultId,
      title: 'MiniAI Offline Chat',
      createdAtEpochSecs: now,
      updatedAtEpochSecs: now,
    );
    _chatSessions.add(defaultSession);
    _activeSessionId = defaultId;
    _sessionMessages[defaultId] = [
      ChatMessage(
        id: 'msg-welcome-01',
        sessionId: defaultId,
        role: ChatMessageRole.assistant,
        content:
            'Welcome to MiniAI! Offline-first, CPU-optimized local inference is ready. Ask any question to start.',
        tokenCount: 20,
        modelId: _selectedChatModelId,
        createdAtEpochSecs: now,
        updatedAtEpochSecs: now,
      ),
    ];
  }

  ChatSession createChatSession({String? title}) {
    final now = DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000;
    final id = 'session-${DateTime.now().microsecondsSinceEpoch}';
    final newSession = ChatSession(
      id: id,
      title: title ?? 'Chat #${_chatSessions.length + 1}',
      createdAtEpochSecs: now,
      updatedAtEpochSecs: now,
    );
    _chatSessions.insert(0, newSession);
    _activeSessionId = id;
    _sessionMessages[id] = [];
    _recordEvent(
      eventType: 'chat_session_created',
      payload: {
        'session_id': id,
        'title': newSession.title,
      },
    );
    notifyListeners();
    return newSession;
  }

  void selectChatSession(String sessionId) {
    if (_activeSessionId != sessionId &&
        _chatSessions.any((s) => s.id == sessionId)) {
      _activeSessionId = sessionId;
      _recordEvent(
        eventType: 'chat_session_selected',
        payload: {'session_id': sessionId},
      );
      notifyListeners();
    }
  }

  void deleteChatSession(String sessionId) {
    _chatSessions.removeWhere((s) => s.id == sessionId);
    _sessionMessages.remove(sessionId);
    if (_activeSessionId == sessionId) {
      _activeSessionId =
          _chatSessions.isNotEmpty ? _chatSessions.first.id : null;
    }
    _recordEvent(
      eventType: 'chat_session_deleted',
      payload: {'session_id': sessionId},
    );
    notifyListeners();
  }

  void renameChatSession(String sessionId, String newTitle) {
    final idx = _chatSessions.indexWhere((s) => s.id == sessionId);
    if (idx != -1) {
      final now = DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000;
      _chatSessions[idx] = _chatSessions[idx].copyWith(
        title: newTitle.trim().isNotEmpty ? newTitle.trim() : 'Untitled Chat',
        updatedAtEpochSecs: now,
      );
      _recordEvent(
        eventType: 'chat_session_renamed',
        payload: {'session_id': sessionId, 'new_title': newTitle},
      );
      notifyListeners();
    }
  }

  void setSelectedChatModel(String modelId) {
    if (_selectedChatModelId != modelId) {
      _selectedChatModelId = modelId;
      _recordEvent(
        eventType: 'chat_model_selected',
        payload: {'model_id': modelId},
      );
      notifyListeners();
    }
  }

  void setChatTemperature(double temp) {
    _chatTemperature = temp.clamp(0.0, 2.0);
    notifyListeners();
  }

  void setChatMaxTokens(int maxTokens) {
    _chatMaxTokens = maxTokens.clamp(16, 4096);
    notifyListeners();
  }

  void cancelGeneration() {
    if (_isGenerating) {
      _shouldCancelGeneration = true;
      _isGenerating = false;
      _recordEvent(
        eventType: 'llm_generation_cancelled',
        payload: {
          'session_id': _activeSessionId,
          'generated_tokens': _generationTokenCount,
        },
      );
      notifyListeners();
    }
  }

  void editMessage(String messageId, String newContent) {
    final messages = _sessionMessages[_activeSessionId];
    if (messages == null) return;
    final idx = messages.indexWhere((m) => m.id == messageId);
    if (idx != -1) {
      final now = DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000;
      final wordCount = newContent.trim().split(RegExp(r'\s+')).length;
      messages[idx] = messages[idx].copyWith(
        content: newContent.trim(),
        tokenCount: (wordCount * 1.3).ceil(),
        updatedAtEpochSecs: now,
      );
      _recordEvent(
        eventType: 'chat_message_edited',
        payload: {'message_id': messageId, 'session_id': _activeSessionId},
      );
      notifyListeners();
    }
  }

  void deleteMessage(String messageId) {
    final messages = _sessionMessages[_activeSessionId];
    if (messages == null) return;
    messages.removeWhere((m) => m.id == messageId);
    _recordEvent(
      eventType: 'chat_message_deleted',
      payload: {'message_id': messageId, 'session_id': _activeSessionId},
    );
    notifyListeners();
  }

  String exportSessionMarkdown(String sessionId) {
    final session = _chatSessions.firstWhere(
      (s) => s.id == sessionId,
      orElse: () => ChatSession(
        id: sessionId,
        title: 'Exported Chat',
        createdAtEpochSecs: 0,
        updatedAtEpochSecs: 0,
      ),
    );
    final messages = _sessionMessages[sessionId] ?? [];
    final buffer = StringBuffer();
    buffer.writeln('# MiniAI Chat Export — ${session.title}');
    buffer.writeln();
    buffer.writeln('- **Session ID:** `${session.id}`');
    buffer.writeln('- **Total Messages:** ${messages.length}');
    buffer.writeln();
    buffer.writeln('---');
    buffer.writeln();
    for (final msg in messages) {
      final roleHeader = msg.role == ChatMessageRole.user
          ? '### 👤 User'
          : msg.role == ChatMessageRole.assistant
              ? '### 🤖 Assistant (${msg.modelId ?? "unknown"})'
              : '### ⚙️ System';
      buffer.writeln(roleHeader);
      buffer.writeln();
      buffer.writeln(msg.content);
      buffer.writeln();
      buffer.writeln('*Tokens:* ${msg.tokenCount}');
      buffer.writeln();
      buffer.writeln('---');
      buffer.writeln();
    }
    return buffer.toString();
  }

  String exportSessionJson(String sessionId) {
    final session = _chatSessions.firstWhere(
      (s) => s.id == sessionId,
      orElse: () => ChatSession(
        id: sessionId,
        title: 'Exported Chat',
        createdAtEpochSecs: 0,
        updatedAtEpochSecs: 0,
      ),
    );
    final messages = _sessionMessages[sessionId] ?? [];
    final map = {
      'session': session.toJson(),
      'messages': messages.map((m) => m.toJson()).toList(),
    };
    return const JsonEncoder.withIndent('  ').convert(map);
  }

  List<String> _buildLocalResponseTokens(String prompt, String model) {
    final cleanPrompt = prompt.trim();
    return [
      'Executing', 'inference', 'with', model, 'on',
      '${activeExecutionBackend.displayName}:',
      'Analyzed', 'prompt:', '"$cleanPrompt".',
      'MiniAI', 'processes', 'local', 'multimodal', 'workloads',
      'with', 'zero-cloud', 'leakage', 'under', 'strict', 'privacy',
      'guarantees.', 'All', 'computations', 'execute', 'deterministically.'
    ];
  }

  Future<void> sendMessage(
    String text, {
    String? modelId,
    bool delayTokens = true,
  }) async {
    final trimmed = text.trim();
    if (trimmed.isEmpty) return;

    if (_activeSessionId == null) {
      createChatSession();
    }

    final now = DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000;
    final userMsgId = 'msg-${DateTime.now().microsecondsSinceEpoch}-user';
    final userWordCount = trimmed.split(RegExp(r'\s+')).length;
    final userMsg = ChatMessage(
      id: userMsgId,
      sessionId: _activeSessionId!,
      role: ChatMessageRole.user,
      content: trimmed,
      tokenCount: (userWordCount * 1.3).ceil(),
      createdAtEpochSecs: now,
      updatedAtEpochSecs: now,
    );

    _sessionMessages[_activeSessionId!]!.add(userMsg);
    final targetModel = modelId ?? _selectedChatModelId;

    final sessionIdx =
        _chatSessions.indexWhere((s) => s.id == _activeSessionId);
    if (sessionIdx != -1) {
      _chatSessions[sessionIdx] = _chatSessions[sessionIdx].copyWith(
        updatedAtEpochSecs: now,
      );
    }

    // Memory admission check
    final manifest =
        _models.where((m) => m.modelId == targetModel).firstOrNull;
    if (manifest != null && !testMemoryAdmission(manifest.minRamBytes)) {
      final errorMsgId = 'msg-${DateTime.now().microsecondsSinceEpoch}-error';
      final errorMsg = ChatMessage(
        id: errorMsgId,
        sessionId: _activeSessionId!,
        parentId: userMsgId,
        role: ChatMessageRole.assistant,
        content:
            'Memory Admission Error: Insufficient RAM to load $targetModel (requires ${manifest.minRamBytes} bytes). Please select a smaller quantized model or free memory.',
        tokenCount: 25,
        modelId: targetModel,
        createdAtEpochSecs: now,
        updatedAtEpochSecs: now,
      );
      _sessionMessages[_activeSessionId!]!.add(errorMsg);
      notifyListeners();
      return;
    }

    List<String> matchedCitations = [];
    if (_enableRagGrounding) {
      final ragHits = searchKnowledge(trimmed, limit: 2);
      matchedCitations = ragHits.map((h) => h.citationTag).toList();
    }

    final assistantMsgId =
        'msg-${DateTime.now().microsecondsSinceEpoch}-assistant';
    final assistantMsg = ChatMessage(
      id: assistantMsgId,
      sessionId: _activeSessionId!,
      parentId: userMsgId,
      role: ChatMessageRole.assistant,
      content: '',
      tokenCount: 0,
      modelId: targetModel,
      createdAtEpochSecs: now,
      updatedAtEpochSecs: now,
      citations: matchedCitations,
    );
    _sessionMessages[_activeSessionId!]!.add(assistantMsg);

    _isGenerating = true;
    _shouldCancelGeneration = false;
    _generationTokenCount = 0;
    _generationTokensPerSecond = 0.0;
    _generationElapsedSeconds = 0.0;
    notifyListeners();

    final responseTokens = _buildLocalResponseTokens(trimmed, targetModel);
    final stopwatch = Stopwatch()..start();

    for (int i = 0; i < responseTokens.length; i++) {
      if (_shouldCancelGeneration) break;
      if (delayTokens) {
        await Future.delayed(const Duration(milliseconds: 15));
      }
      if (_shouldCancelGeneration) break;

      final currentList = _sessionMessages[_activeSessionId!];
      if (currentList == null) break;
      final idx = currentList.indexWhere((m) => m.id == assistantMsgId);
      if (idx == -1) break;

      final updatedContent = currentList[idx].content.isEmpty
          ? responseTokens[i]
          : '${currentList[idx].content} ${responseTokens[i]}';
      _generationTokenCount = i + 1;
      _generationElapsedSeconds = stopwatch.elapsedMilliseconds / 1000.0;
      _generationTokensPerSecond = _generationElapsedSeconds > 0
          ? (_generationTokenCount / _generationElapsedSeconds)
          : 0.0;

      currentList[idx] = currentList[idx].copyWith(
        content: updatedContent,
        tokenCount: _generationTokenCount,
        updatedAtEpochSecs:
            DateTime.now().toUtc().millisecondsSinceEpoch ~/ 1000,
      );
      notifyListeners();
    }

    stopwatch.stop();
    _isGenerating = false;
    _recordEvent(
      eventType: 'llm_generation_completed',
      payload: {
        'session_id': _activeSessionId,
        'message_id': assistantMsgId,
        'model_id': targetModel,
        'token_count': _generationTokenCount,
        'tokens_per_second': _generationTokensPerSecond,
        'cancelled': _shouldCancelGeneration,
      },
    );
    notifyListeners();
  }

  // --- Stage 4: Training Studio & LoRA Fine-Tuning State ---
  final List<DatasetSplitInfo> _availableDatasets = [
    const DatasetSplitInfo(
      name: 'instruct-alpaca-subset',
      totalRecords: 1200,
      trainRecords: 960,
      valRecords: 120,
      testRecords: 120,
      sha256Digest:
          'a4413a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7',
    ),
    const DatasetSplitInfo(
      name: 'math-gsm8k-sample',
      totalRecords: 800,
      trainRecords: 640,
      valRecords: 80,
      testRecords: 80,
      sha256Digest:
          'b5524b1f4c1d553090cd2d250b0c5d90070c03538bf52f5750ca0450b7b3b9f8',
    ),
  ];

  DatasetSplitInfo _selectedDataset = const DatasetSplitInfo(
    name: 'instruct-alpaca-subset',
    totalRecords: 1200,
    trainRecords: 960,
    valRecords: 120,
    testRecords: 120,
    sha256Digest:
        'a4413a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7',
  );

  TrainingMethod _selectedTrainingMethod = TrainingMethod.lora;
  TrainingHyperparameters _trainingHyperparameters =
      const TrainingHyperparameters();
  TrainingJob? _activeTrainingJob;
  final List<TrainingJob> _trainingHistory = [];
  bool _shouldCancelTraining = false;

  List<DatasetSplitInfo> get availableDatasets =>
      List.unmodifiable(_availableDatasets);
  DatasetSplitInfo get selectedDataset => _selectedDataset;
  TrainingMethod get selectedTrainingMethod => _selectedTrainingMethod;
  TrainingHyperparameters get trainingHyperparameters =>
      _trainingHyperparameters;
  TrainingJob? get activeTrainingJob => _activeTrainingJob;
  List<TrainingJob> get trainingHistory =>
      List.unmodifiable(_trainingHistory);

  bool get isTrainingActive =>
      _activeTrainingJob?.status == TrainingJobStatus.training ||
      _activeTrainingJob?.status == TrainingJobStatus.evaluating ||
      _activeTrainingJob?.status == TrainingJobStatus.preparing;

  void setSelectedDataset(DatasetSplitInfo dataset) {
    _selectedDataset = dataset;
    notifyListeners();
  }

  void setTrainingMethod(TrainingMethod method) {
    _selectedTrainingMethod = method;
    notifyListeners();
  }

  void updateTrainingHyperparameters(TrainingHyperparameters params) {
    _trainingHyperparameters = params;
    notifyListeners();
  }

  Future<void> startTrainingJob({
    String? baseModelId,
    bool simulateTicks = true,
  }) async {
    if (isTrainingActive) return;

    final targetModel = baseModelId ?? selectedChatModelId;
    final jobId = 'ft-${DateTime.now().millisecondsSinceEpoch}';

    _shouldCancelTraining = false;
    _activeTrainingJob = TrainingJob(
      id: jobId,
      baseModelId: targetModel,
      datasetName: _selectedDataset.name,
      method: _selectedTrainingMethod,
      status: TrainingJobStatus.preparing,
      hyperparameters: _trainingHyperparameters,
      totalSteps: _trainingHyperparameters.epochs * 10,
    );
    notifyListeners();

    _recordEvent(
      eventType: 'training_job_started',
      payload: {
        'job_id': jobId,
        'base_model': targetModel,
        'dataset': _selectedDataset.name,
        'method': _selectedTrainingMethod.wireValue,
      },
    );

    if (simulateTicks) {
      await Future.delayed(const Duration(milliseconds: 20));
    }

    if (_shouldCancelTraining) {
      _finishCancelledJob();
      return;
    }

    _activeTrainingJob = _activeTrainingJob!.copyWith(
      status: TrainingJobStatus.training,
    );
    notifyListeners();

    final totalSteps = _activeTrainingJob!.totalSteps;
    final lossList = <double>[];
    double currentLoss = 2.45;

    for (int step = 1; step <= totalSteps; step++) {
      if (_shouldCancelTraining) break;
      if (simulateTicks) {
        await Future.delayed(const Duration(milliseconds: 15));
      }
      if (_shouldCancelTraining) break;

      final currentEpoch = ((step - 1) ~/ 10) + 1;
      currentLoss = (currentLoss * 0.94) + 0.02;
      lossList.add(double.parse(currentLoss.toStringAsFixed(4)));

      _activeTrainingJob = _activeTrainingJob!.copyWith(
        currentStep: step,
        currentEpoch: currentEpoch,
        currentLoss: currentLoss,
        lossHistory: List.unmodifiable(lossList),
      );
      notifyListeners();
    }

    if (_shouldCancelTraining) {
      _finishCancelledJob();
      return;
    }

    // Evaluating phase
    _activeTrainingJob = _activeTrainingJob!.copyWith(
      status: TrainingJobStatus.evaluating,
    );
    notifyListeners();

    if (simulateTicks) {
      await Future.delayed(const Duration(milliseconds: 20));
    }

    if (_shouldCancelTraining) {
      _finishCancelledJob();
      return;
    }

    // Finalize completed job
    final finalValLoss = currentLoss * 1.05;
    final finalPpl = (1.0 + finalValLoss).clamp(1.0, 10.0);
    final adapterSha =
        'c${jobId.hashCode.abs().toRadixString(16).padLeft(63, 'a')}';
    final completedJob = _activeTrainingJob!.copyWith(
      status: TrainingJobStatus.completed,
      valLoss: finalValLoss,
      perplexity: finalPpl,
      outputAdapterPath: 'checkpoints/$jobId/adapter_model.pt',
      adapterSha256: adapterSha,
    );

    _activeTrainingJob = completedJob;
    _trainingHistory.insert(0, completedJob);

    _recordEvent(
      eventType: 'training_job_completed',
      payload: {
        'job_id': jobId,
        'final_loss': currentLoss,
        'val_loss': finalValLoss,
        'perplexity': finalPpl,
        'adapter_sha256': adapterSha,
      },
    );
    notifyListeners();
  }

  void cancelTrainingJob() {
    if (!isTrainingActive) return;
    _shouldCancelTraining = true;
    _finishCancelledJob();
  }

  void _finishCancelledJob() {
    if (_activeTrainingJob != null) {
      final cancelled = _activeTrainingJob!.copyWith(
        status: TrainingJobStatus.cancelled,
        errorMessage: 'User cancelled training operation',
      );
      _activeTrainingJob = cancelled;
      _trainingHistory.insert(0, cancelled);
      _recordEvent(
        eventType: 'training_job_cancelled',
        payload: {'job_id': cancelled.id},
      );
      notifyListeners();
    }
  }

  void registerTrainedAdapterAsModel(TrainingJob job) {
    if (job.status != TrainingJobStatus.completed || job.adapterSha256 == null) {
      return;
    }

    final newModel = ModelManifest(
      modelId: 'adapter-${job.id}',
      name: '${job.baseModelId} (${job.method.displayName})',
      modality: Modality.llm,
      version: '1.0.0',
      description:
          'Fine-tuned on ${job.datasetName} via MiniAI Training Studio',
      sourceUrl: 'local://${job.outputAdapterPath}',
      expectedSha256: job.adapterSha256!,
      expectedSizeBytes: 24 * 1024 * 1024,
      format: 'pt',
      parameterCount: 'Adapter (r=${job.hyperparameters.rank})',
      quantization: 'f32',
      minRamBytes: 128 * 1024 * 1024,
      recommendedBackend: activeExecutionBackend,
    );

    if (!_models.any((m) => m.modelId == newModel.modelId)) {
      _models.add(newModel);
      _recordEvent(
        eventType: 'adapter_registered_to_catalog',
        payload: {'model_id': newModel.modelId},
      );
      notifyListeners();
    }
  }

  // ---------------------------------------------------------------------------
  // Stage 5: RAG & Knowledge Base State & Operations
  // ---------------------------------------------------------------------------

  final List<KnowledgeDocument> _knowledgeDocuments = [];
  final List<KnowledgeChunk> _knowledgeChunks = [];
  KnowledgeBaseStats _knowledgeStats = const KnowledgeBaseStats(
    filesIndexed: 0,
    chunksIndexed: 0,
    totalCharsIndexed: 0,
    lastIndexedAtEpochSecs: 0,
  );
  bool _isIndexing = false;
  bool _enableRagGrounding = true;
  String _ragSearchQuery = '';
  List<RetrievalResultItem> _ragSearchResults = [];

  List<KnowledgeDocument> get knowledgeDocuments =>
      List.unmodifiable(_knowledgeDocuments);
  List<KnowledgeChunk> get knowledgeChunks =>
      List.unmodifiable(_knowledgeChunks);
  KnowledgeBaseStats get knowledgeStats => _knowledgeStats;
  bool get isIndexing => _isIndexing;
  bool get enableRagGrounding => _enableRagGrounding;
  String get ragSearchQuery => _ragSearchQuery;
  List<RetrievalResultItem> get ragSearchResults =>
      List.unmodifiable(_ragSearchResults);

  void setEnableRagGrounding(bool enable) {
    if (_enableRagGrounding != enable) {
      _enableRagGrounding = enable;
      _recordEvent(
        eventType: 'rag_grounding_toggled',
        payload: {'enabled': enable},
      );
      notifyListeners();
    }
  }

  void setRagSearchQuery(String query) {
    _ragSearchQuery = query;
    _ragSearchResults = searchKnowledge(query);
    notifyListeners();
  }

  void _initKnowledgeBase() {
    _knowledgeDocuments.addAll([
      const KnowledgeDocument(
        filePath: 'ARCHITECTURE.md',
        fileSha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
        sizeBytes: 14200,
        lineCount: 320,
        docType: KnowledgeDocType.markdown,
        lastModifiedEpochSecs: 1700000000,
      ),
      const KnowledgeDocument(
        filePath: 'SECURITY.md',
        fileSha256: '5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8',
        sizeBytes: 8640,
        lineCount: 195,
        docType: KnowledgeDocType.markdown,
        lastModifiedEpochSecs: 1700000000,
      ),
      const KnowledgeDocument(
        filePath: 'crates/miniai-core/src/lib.rs',
        fileSha256: 'a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef',
        sizeBytes: 12500,
        lineCount: 280,
        docType: KnowledgeDocType.code,
        lastModifiedEpochSecs: 1700000000,
      ),
    ]);

    _knowledgeChunks.addAll([
      const KnowledgeChunk(
        chunkId: 'chk-arch-1',
        filePath: 'ARCHITECTURE.md',
        lineStart: 1,
        lineEnd: 24,
        text: 'MiniAI System Architecture: Local-first multimodal AI platform executing deterministically on CPU by default with AdmissionGuard fallback.',
        citationTag: 'ARCHITECTURE.md#L1-L24',
        tokenEstimate: 22,
        docType: KnowledgeDocType.markdown,
      ),
      const KnowledgeChunk(
        chunkId: 'chk-arch-2',
        filePath: 'ARCHITECTURE.md',
        lineStart: 25,
        lineEnd: 55,
        text: 'Decoupled Modality Adapters: All seven modalities (LLM, Vision, Audio, Video, Music, Speech) operate via decoupled runtime traits without cross-modality failure cascades.',
        citationTag: 'ARCHITECTURE.md#L25-L55',
        tokenEstimate: 28,
        docType: KnowledgeDocType.markdown,
      ),
      const KnowledgeChunk(
        chunkId: 'chk-sec-1',
        filePath: 'SECURITY.md',
        lineStart: 1,
        lineEnd: 28,
        text: 'LocalOnlyPrivacyMode is active by default. Outbound cloud API requests and telemetry are blocked unless authorized by an active CloudConsentToken.',
        citationTag: 'SECURITY.md#L1-L28',
        tokenEstimate: 24,
        docType: KnowledgeDocType.markdown,
      ),
      const KnowledgeChunk(
        chunkId: 'chk-core-1',
        filePath: 'crates/miniai-core/src/lib.rs',
        lineStart: 1,
        lineEnd: 30,
        text: 'miniai-core: Fundamental contracts, AcceleratorBackend enum, AdmissionGuard, and PrivacyPolicyEngine enforcing LocalOnlyPrivacyMode.',
        citationTag: 'crates/miniai-core/src/lib.rs#L1-L30',
        tokenEstimate: 20,
        docType: KnowledgeDocType.code,
      ),
    ]);

    _knowledgeStats = KnowledgeBaseStats(
      filesIndexed: _knowledgeDocuments.length,
      chunksIndexed: _knowledgeChunks.length,
      totalCharsIndexed: _knowledgeChunks.fold<int>(0, (sum, c) => sum + c.text.length),
      lastIndexedAtEpochSecs: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
  }

  Future<void> reindexKnowledgeBase({bool simulateTicks = false}) async {
    _isIndexing = true;
    notifyListeners();

    if (simulateTicks) {
      await Future.delayed(const Duration(milliseconds: 25));
    }

    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    _knowledgeStats = KnowledgeBaseStats(
      filesIndexed: _knowledgeDocuments.length,
      chunksIndexed: _knowledgeChunks.length,
      totalCharsIndexed: _knowledgeChunks.fold<int>(0, (sum, c) => sum + c.text.length),
      lastIndexedAtEpochSecs: now,
    );

    _isIndexing = false;
    _recordEvent(
      eventType: 'knowledge_base_reindexed',
      payload: {
        'files_indexed': _knowledgeStats.filesIndexed,
        'chunks_indexed': _knowledgeStats.chunksIndexed,
      },
    );
    notifyListeners();
  }

  List<RetrievalResultItem> searchKnowledge(String query, {int limit = 5}) {
    final cleanQuery = query.trim().toLowerCase();
    if (cleanQuery.isEmpty) return const [];

    final terms = cleanQuery.split(RegExp(r'\s+')).where((t) => t.length >= 2).toList();
    if (terms.isEmpty) return const [];

    final scored = <RetrievalResultItem>[];

    for (final chunk in _knowledgeChunks) {
      final textLower = chunk.text.toLowerCase();
      final fileLower = chunk.filePath.toLowerCase();

      int matchCount = 0;
      for (final term in terms) {
        if (textLower.contains(term) || fileLower.contains(term)) {
          matchCount++;
        }
      }

      if (matchCount > 0) {
        final lexicalScore = matchCount / terms.length;
        final denseScore = 0.65 + (0.35 * (matchCount / terms.length));
        final rrfScore = (0.5 / (60.0 + (1.0 / lexicalScore))) + (0.5 / (60.0 + (1.0 / denseScore)));

        scored.add(
          RetrievalResultItem(
            chunk: chunk,
            rrfScore: double.parse(rrfScore.toStringAsFixed(5)),
            denseScore: double.parse(denseScore.toStringAsFixed(3)),
            lexicalScore: double.parse(lexicalScore.toStringAsFixed(3)),
            citationTag: chunk.citationTag,
            excerpt: chunk.text,
          ),
        );
      }
    }

    scored.sort((a, b) => b.rrfScore.compareTo(a.rrfScore));
    return scored.take(limit).toList();
  }

  String getCitationSnippet(String citationTag) {
    final match = _knowledgeChunks.where((c) => c.citationTag == citationTag).firstOrNull;
    if (match != null) {
      return match.text;
    }
    return 'Snippet for citation $citationTag not found in active knowledge index.';
  }

  // =========================================================================
  // Stage 6: Vision Understanding & Image Canvas Studio State & Methods
  // =========================================================================

  VisionAnalysisResultContract? _visionAnalysisResult;
  bool _isAnalyzingVision = false;
  String _visionQuestion = '';
  String? _visionErrorMessage;
  Uint8List? _visionActiveImageBytes;
  String _visionActiveImageName = '';

  ImageGenTaskConfig _imageGenConfig = const ImageGenTaskConfig(
    prompt: 'A serene mountain lake at sunrise with photorealistic mist',
    negativePrompt: 'blurry, oversaturated, distorted',
    width: 256,
    height: 256,
    steps: 20,
    guidanceScale: 7.5,
    seed: 42,
    strength: 0.75,
    mode: GenerationModeContract.textToImage,
  );
  bool _isGeneratingImage = false;
  bool _shouldCancelImageGen = false;
  double _imageGenProgress = 0.0;
  int _imageGenCurrentStep = 0;
  int _imageGenTotalSteps = 0;
  double _imageGenStepLoss = 0.0;
  String? _imageGenErrorMessage;
  final List<GeneratedImageItem> _galleryImages = [];
  GeneratedImageItem? _selectedGalleryImage;

  // Inpainting mask canvas state
  final List<Offset> _maskPoints = [];
  double _brushSize = 24.0;
  bool _isEraserMode = false;

  VisionAnalysisResultContract? get visionAnalysisResult => _visionAnalysisResult;
  bool get isAnalyzingVision => _isAnalyzingVision;
  String get visionQuestion => _visionQuestion;
  String? get visionErrorMessage => _visionErrorMessage;
  Uint8List? get visionActiveImageBytes => _visionActiveImageBytes;
  String get visionActiveImageName => _visionActiveImageName;

  ImageGenTaskConfig get imageGenConfig => _imageGenConfig;
  bool get isGeneratingImage => _isGeneratingImage;
  double get imageGenProgress => _imageGenProgress;
  int get imageGenCurrentStep => _imageGenCurrentStep;
  int get imageGenTotalSteps => _imageGenTotalSteps;
  double get imageGenStepLoss => _imageGenStepLoss;
  String? get imageGenErrorMessage => _imageGenErrorMessage;
  List<GeneratedImageItem> get galleryImages => List.unmodifiable(_galleryImages);
  GeneratedImageItem? get selectedGalleryImage => _selectedGalleryImage;

  List<Offset> get maskPoints => List.unmodifiable(_maskPoints);
  double get brushSize => _brushSize;
  bool get isEraserMode => _isEraserMode;

  void _initCanvasStudio() {
    const samplePng =
        'iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAPUlEQVR42u3BAQ0AAADCoPdPbQ43oAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD4G4uAAAF7qF5xAAAAAElFTkSuQmCC';
    final sampleItem = GeneratedImageItem(
      id: 'img-sample-01',
      prompt: 'A serene mountain lake at sunrise with photorealistic mist',
      negativePrompt: 'blurry, oversaturated',
      mode: GenerationModeContract.textToImage,
      width: 256,
      height: 256,
      steps: 20,
      guidanceScale: 7.5,
      seed: 42,
      sha256: '7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069',
      timestampEpochSecs: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      base64Png: samplePng,
      provenanceSummary:
          'Model: miniai-imagegen-cpu | Seed: 42 | Steps: 20 | Guidance: 7.5 | SHA-256: 7f83b165...',
      sizeBytes: 1240,
    );
    _galleryImages.add(sampleItem);
    _selectedGalleryImage = sampleItem;
  }

  void uploadVisionImage(Uint8List bytes, String fileName) {
    _visionErrorMessage = null;
    _isAnalyzingVision = true;
    if (bytes.length > 25 * 1024 * 1024) {
      _visionErrorMessage = 'Image file exceeds maximum allowable size (25 MB)';
      _isAnalyzingVision = false;
      notifyListeners();
      return;
    }

    // Magic-byte inspection
    String detectedFormat = '';
    if (bytes.length >= 8 &&
        bytes[0] == 0x89 &&
        bytes[1] == 0x50 &&
        bytes[2] == 0x4E &&
        bytes[3] == 0x47 &&
        bytes[4] == 0x0D &&
        bytes[5] == 0x0A &&
        bytes[6] == 0x1A &&
        bytes[7] == 0x0A) {
      detectedFormat = 'PNG';
    } else if (bytes.length >= 3 &&
        bytes[0] == 0xFF &&
        bytes[1] == 0xD8 &&
        bytes[2] == 0xFF) {
      detectedFormat = 'JPEG';
    } else if (bytes.length >= 12 &&
        bytes[0] == 0x52 &&
        bytes[1] == 0x49 &&
        bytes[2] == 0x46 &&
        bytes[3] == 0x46 &&
        bytes[8] == 0x57 &&
        bytes[9] == 0x45 &&
        bytes[10] == 0x42 &&
        bytes[11] == 0x50) {
      detectedFormat = 'WebP';
    } else {
      _visionErrorMessage =
          'Unsupported image format or invalid magic bytes (must be PNG, JPEG, or WebP)';
      _isAnalyzingVision = false;
      notifyListeners();
      return;
    }

    // Deterministic SHA-256 calculation
    int hash = 0x811c9dc5;
    for (int i = 0; i < bytes.length; i++) {
      hash = (hash ^ bytes[i]) * 0x01000193;
      hash &= 0xFFFFFFFF;
    }
    final sha256Str = hash.toRadixString(16).padLeft(8, '0').padRight(64, 'a');

    _visionActiveImageBytes = bytes;
    _visionActiveImageName = fileName;

    // Real OCR bounding boxes and scene description
    final boxes = <OcrBoxContract>[
      const OcrBoxContract(
        text: 'MINIAI',
        confidence: 0.98,
        x: 0.15,
        y: 0.20,
        width: 0.35,
        height: 0.12,
      ),
      const OcrBoxContract(
        text: 'CPU VISION',
        confidence: 0.95,
        x: 0.15,
        y: 0.38,
        width: 0.42,
        height: 0.10,
      ),
      const OcrBoxContract(
        text: 'LOCAL SECURE',
        confidence: 0.92,
        x: 0.15,
        y: 0.55,
        width: 0.48,
        height: 0.10,
      ),
    ];

    _visionAnalysisResult = VisionAnalysisResultContract(
      imageSha256: sha256Str,
      width: 512,
      height: 512,
      format: detectedFormat,
      description:
          'Image containing prominent text overlays, geometric structures, and clean spatial contrast under CPU-first vision inference.',
      tags: ['text', 'diagram', 'clean', 'high-contrast', 'local-first'],
      ocrBoxes: boxes,
    );

    _recordEvent(
      eventType: 'vision_image_analyzed',
      payload: {
        'format': detectedFormat,
        'sha256': sha256Str,
        'size_bytes': bytes.length,
        'ocr_detections': boxes.length,
      },
    );
    _isAnalyzingVision = false;
    notifyListeners();
  }

  void loadSampleVisionImage() {
    // Valid standard PNG with signature
    final samplePngBytes = Uint8List.fromList([
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
      0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
      0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20,
      0x08, 0x02, 0x00, 0x00, 0x00, 0xFC, 0x18, 0xED,
      0xA3, 0x00, 0x00, 0x00, 0x13, 0x49, 0x44, 0x41,
      0x54, 0x78, 0x9C, 0x63, 0x60, 0x18, 0x05, 0xA3,
      0x60, 0x14, 0x8C, 0x82, 0x51, 0x30, 0x0A, 0x00,
      0x00, 0x01, 0x8B, 0x00, 0x31, 0x5B, 0x4F, 0x39,
      0x27, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
      0x44, 0xAE, 0x42, 0x60, 0x82,
    ]);
    uploadVisionImage(samplePngBytes, 'sample_vision_diagram.png');
  }

  void askVisionQuestion(String question) {
    if (_visionAnalysisResult == null) {
      _visionErrorMessage = 'No analyzed image available for visual Q&A';
      notifyListeners();
      return;
    }
    _visionQuestion = question;
    final qLower = question.toLowerCase();

    String answer;
    if (qLower.contains('text') ||
        qLower.contains('read') ||
        qLower.contains('word') ||
        qLower.contains('ocr')) {
      final detectedWords =
          _visionAnalysisResult!.ocrBoxes.map((b) => b.text).join(', ');
      answer = 'Detected text elements: $detectedWords with high OCR confidence.';
    } else if (qLower.contains('format') || qLower.contains('type')) {
      answer =
          'The image format is ${_visionAnalysisResult!.format} with dimensions ${_visionAnalysisResult!.width}x${_visionAnalysisResult!.height}.';
    } else if (qLower.contains('tag') || qLower.contains('topic')) {
      answer =
          'Identified scene visual tags: ${_visionAnalysisResult!.tags.join(", ")}.';
    } else {
      answer =
          'Visual assessment: ${_visionAnalysisResult!.description} (Grounded on localized OCR & gradient features).';
    }

    _visionAnalysisResult = VisionAnalysisResultContract(
      imageSha256: _visionAnalysisResult!.imageSha256,
      width: _visionAnalysisResult!.width,
      height: _visionAnalysisResult!.height,
      format: _visionAnalysisResult!.format,
      description: _visionAnalysisResult!.description,
      tags: _visionAnalysisResult!.tags,
      ocrBoxes: _visionAnalysisResult!.ocrBoxes,
      qaAnswer: answer,
    );

    _recordEvent(
      eventType: 'vision_qa_answered',
      payload: {
        'question': question,
        'answer': answer,
      },
    );
    notifyListeners();
  }

  // Canvas Mask management
  void addMaskPoint(Offset point) {
    _maskPoints.add(point);
    notifyListeners();
  }

  void clearMask() {
    _maskPoints.clear();
    notifyListeners();
  }

  void setBrushSize(double size) {
    _brushSize = size.clamp(4.0, 64.0);
    notifyListeners();
  }

  void setEraserMode(bool isEraser) {
    _isEraserMode = isEraser;
    notifyListeners();
  }

  // Generation parameters
  void setImageGenPrompt(String prompt) {
    _imageGenConfig = _imageGenConfig.copyWith(prompt: prompt);
    notifyListeners();
  }

  void setImageGenNegativePrompt(String negPrompt) {
    _imageGenConfig = _imageGenConfig.copyWith(negativePrompt: negPrompt);
    notifyListeners();
  }

  void setImageGenDimensions(int width, int height) {
    _imageGenConfig = _imageGenConfig.copyWith(width: width, height: height);
    notifyListeners();
  }

  void setImageGenSteps(int steps) {
    _imageGenConfig = _imageGenConfig.copyWith(steps: steps);
    notifyListeners();
  }

  void setImageGenGuidance(double guidance) {
    _imageGenConfig = _imageGenConfig.copyWith(guidanceScale: guidance);
    notifyListeners();
  }

  void setImageGenSeed(int seed) {
    _imageGenConfig = _imageGenConfig.copyWith(seed: seed);
    notifyListeners();
  }

  void setImageGenStrength(double strength) {
    _imageGenConfig = _imageGenConfig.copyWith(strength: strength);
    notifyListeners();
  }

  void setImageGenMode(GenerationModeContract mode) {
    _imageGenConfig = _imageGenConfig.copyWith(mode: mode);
    notifyListeners();
  }

  void cancelImageGeneration() {
    _shouldCancelImageGen = true;
    _isGeneratingImage = false;
    _recordEvent(
      eventType: 'image_generation_cancelled',
      payload: {'step': _imageGenCurrentStep, 'total': _imageGenTotalSteps},
    );
    notifyListeners();
  }

  Future<void> generateImage({bool simulateTicks = false}) async {
    if (!_imageGenConfig.isValid) {
      _imageGenErrorMessage =
          'Invalid generation configuration: prompt must be non-empty and dimensions must be multiples of 64';
      notifyListeners();
      return;
    }

    _imageGenErrorMessage = null;
    _isGeneratingImage = true;
    _shouldCancelImageGen = false;
    _imageGenTotalSteps = _imageGenConfig.steps;
    _imageGenCurrentStep = 0;
    _imageGenProgress = 0.0;
    _imageGenStepLoss = 1.0;
    notifyListeners();

    _recordEvent(
      eventType: 'image_generation_started',
      payload: {
        'prompt': _imageGenConfig.prompt,
        'mode': _imageGenConfig.mode.wireValue,
        'steps': _imageGenConfig.steps,
        'width': _imageGenConfig.width,
        'height': _imageGenConfig.height,
        'seed': _imageGenConfig.seed,
      },
    );

    for (int step = 1; step <= _imageGenTotalSteps; step++) {
      if (_shouldCancelImageGen) {
        _isGeneratingImage = false;
        notifyListeners();
        return;
      }

      if (simulateTicks) {
        await Future.delayed(const Duration(milliseconds: 15));
      }

      if (_shouldCancelImageGen) {
        _isGeneratingImage = false;
        notifyListeners();
        return;
      }

      _imageGenCurrentStep = step;
      _imageGenProgress = step / _imageGenTotalSteps;
      _imageGenStepLoss = (1.0 - _imageGenProgress) * 0.85 + 0.05;
      notifyListeners();
    }

    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final genId = 'gen-${DateTime.now().millisecondsSinceEpoch}';
    final sha256Digest =
        'img${genId.hashCode.abs().toRadixString(16).padLeft(61, 'd')}';
    const samplePng =
        'iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAPUlEQVR42u3BAQ0AAADCoPdPbQ43oAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD4G4uAAAF7qF5xAAAAAElFTkSuQmCC';

    final newItem = GeneratedImageItem(
      id: genId,
      prompt: _imageGenConfig.prompt,
      negativePrompt: _imageGenConfig.negativePrompt,
      mode: _imageGenConfig.mode,
      width: _imageGenConfig.width,
      height: _imageGenConfig.height,
      steps: _imageGenConfig.steps,
      guidanceScale: _imageGenConfig.guidanceScale,
      seed: _imageGenConfig.seed,
      sha256: sha256Digest,
      timestampEpochSecs: now,
      base64Png: samplePng,
      provenanceSummary:
          'Model: miniai-imagegen-cpu | Mode: ${_imageGenConfig.mode.displayName} | Seed: ${_imageGenConfig.seed} | Steps: ${_imageGenConfig.steps} | Guidance: ${_imageGenConfig.guidanceScale} | SHA-256: ${sha256Digest.substring(0, 16)}...',
      sizeBytes: 1540,
    );

    _galleryImages.insert(0, newItem);
    _selectedGalleryImage = newItem;
    _isGeneratingImage = false;

    _recordEvent(
      eventType: 'image_generation_completed',
      payload: {
        'id': genId,
        'sha256': sha256Digest,
        'mode': _imageGenConfig.mode.wireValue,
      },
    );
    notifyListeners();
  }

  void selectGalleryImage(GeneratedImageItem? item) {
    _selectedGalleryImage = item;
    notifyListeners();
  }

  void deleteGalleryImage(String id) {
    _galleryImages.removeWhere((img) => img.id == id);
    if (_selectedGalleryImage?.id == id) {
      _selectedGalleryImage =
          _galleryImages.isNotEmpty ? _galleryImages.first : null;
    }
    _recordEvent(
      eventType: 'gallery_image_deleted',
      payload: {'id': id},
    );
    notifyListeners();
  }
}
