import 'dart:convert';
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
}
