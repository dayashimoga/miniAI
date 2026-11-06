/// Cross-language contract models mirroring `crates/miniai-core/src/lib.rs`.
library;

enum Modality {
  llm('llm', 'LLM Text & Reasoning'),
  vision('vision', 'Vision Understanding'),
  imageGeneration('image_generation', 'Image Generation & Editing'),
  speechAsr('speech_asr', 'Speech Recognition (ASR)'),
  speechTts('speech_tts', 'Text-to-Speech (TTS)'),
  music('music', 'Music & Audio Synthesis'),
  video('video', 'Video Understanding & Generation');

  const Modality(this.wireValue, this.displayName);

  final String wireValue;
  final String displayName;

  static Modality fromWire(String value) {
    for (final modality in Modality.values) {
      if (modality.wireValue == value) {
        return modality;
      }
    }
    throw ArgumentError.value(value, 'value', 'Unknown Modality wire value');
  }
}

enum AcceleratorBackend {
  cpu('cpu', 'CPU (Universal Baseline)'),
  cuda('cuda', 'NVIDIA CUDA'),
  vulkan('vulkan', 'Vulkan Compute'),
  metal('metal', 'Apple Metal'),
  directMl('direct_ml', 'DirectML'),
  rocm('rocm', 'AMD ROCm'),
  coreMl('core_ml', 'Apple CoreML'),
  nnapi('nnapi', 'Android NNAPI');

  const AcceleratorBackend(this.wireValue, this.displayName);

  final String wireValue;
  final String displayName;

  bool get isCpu => this == AcceleratorBackend.cpu;

  static AcceleratorBackend fromWire(String value) {
    for (final backend in AcceleratorBackend.values) {
      if (backend.wireValue == value) {
        return backend;
      }
    }
    throw ArgumentError.value(
      value,
      'value',
      'Unknown AcceleratorBackend wire value',
    );
  }
}

class CpuFeatures {
  const CpuFeatures({
    this.avx2 = false,
    this.avx512f = false,
    this.avx512bw = false,
    this.avx512cd = false,
    this.avx512dq = false,
    this.avx512vl = false,
    this.fma = false,
    this.sse4_2 = false,
    this.neon = false,
    this.sve = false,
    this.dotprod = false,
  });

  final bool avx2;
  final bool avx512f;
  final bool avx512bw;
  final bool avx512cd;
  final bool avx512dq;
  final bool avx512vl;
  final bool fma;
  final bool sse4_2;
  final bool neon;
  final bool sve;
  final bool dotprod;

  List<String> get enabledFeatures {
    final list = <String>[];
    if (avx2) list.add('AVX2');
    if (avx512f) list.add('AVX512F');
    if (avx512bw) list.add('AVX512BW');
    if (avx512cd) list.add('AVX512CD');
    if (avx512dq) list.add('AVX512DQ');
    if (avx512vl) list.add('AVX512VL');
    if (fma) list.add('FMA');
    if (sse4_2) list.add('SSE4.2');
    if (neon) list.add('NEON');
    if (sve) list.add('SVE');
    if (dotprod) list.add('DotProd');
    return list;
  }

  Map<String, Object?> toJson() => {
        'avx2': avx2,
        'avx512f': avx512f,
        'avx512bw': avx512bw,
        'avx512cd': avx512cd,
        'avx512dq': avx512dq,
        'avx512vl': avx512vl,
        'fma': fma,
        'sse4_2': sse4_2,
        'neon': neon,
        'sve': sve,
        'dotprod': dotprod,
      };

  factory CpuFeatures.fromJson(Map<String, Object?> json) {
    return CpuFeatures(
      avx2: json['avx2'] as bool? ?? false,
      avx512f: json['avx512f'] as bool? ?? false,
      avx512bw: json['avx512bw'] as bool? ?? false,
      avx512cd: json['avx512cd'] as bool? ?? false,
      avx512dq: json['avx512dq'] as bool? ?? false,
      avx512vl: json['avx512vl'] as bool? ?? false,
      fma: json['fma'] as bool? ?? false,
      sse4_2: json['sse4_2'] as bool? ?? false,
      neon: json['neon'] as bool? ?? false,
      sve: json['sve'] as bool? ?? false,
      dotprod: json['dotprod'] as bool? ?? false,
    );
  }
}

class GpuDevice {
  const GpuDevice({
    required this.name,
    required this.backend,
    required this.totalVramBytes,
    required this.availableVramBytes,
  });

  final String name;
  final AcceleratorBackend backend;
  final int totalVramBytes;
  final int availableVramBytes;

  int get safeVramLimitBytes => (availableVramBytes ~/ 100) * 85;

  bool canAdmitVram(int requiredBytes) {
    return requiredBytes == 0 || requiredBytes <= safeVramLimitBytes;
  }

  Map<String, Object?> toJson() => {
        'name': name,
        'backend': backend.wireValue,
        'total_vram_bytes': totalVramBytes,
        'available_vram_bytes': availableVramBytes,
      };

  factory GpuDevice.fromJson(Map<String, Object?> json) {
    return GpuDevice(
      name: json['name'] as String? ?? 'Generic GPU',
      backend: AcceleratorBackend.fromWire(json['backend'] as String? ?? 'cpu'),
      totalVramBytes: json['total_vram_bytes'] as int? ?? 0,
      availableVramBytes: json['available_vram_bytes'] as int? ?? 0,
    );
  }
}

class HardwareProfile {
  const HardwareProfile({
    required this.osFamily,
    required this.arch,
    required this.logicalCpuCores,
    required this.totalMemoryBytes,
    required this.availableMemoryBytes,
    required this.detectedBackends,
    this.cpuFeatures = const CpuFeatures(),
    this.gpus = const [],
  });

  final String osFamily;
  final String arch;
  final int logicalCpuCores;
  final int totalMemoryBytes;
  final int availableMemoryBytes;
  final List<AcceleratorBackend> detectedBackends;
  final CpuFeatures cpuFeatures;
  final List<GpuDevice> gpus;

  /// Returns 85% of available memory as the safe admission limit.
  int get safeMemoryLimitBytes => (availableMemoryBytes ~/ 100) * 85;

  bool canAdmitModel(int requiredBytes) {
    return requiredBytes > 0 && requiredBytes <= safeMemoryLimitBytes;
  }

  AcceleratorBackend selectExecutionBackend({
    AcceleratorBackend? preferred,
    required bool forceCpu,
  }) {
    if (forceCpu) {
      return AcceleratorBackend.cpu;
    }
    if (preferred != null && detectedBackends.contains(preferred)) {
      return preferred;
    }
    for (final backend in detectedBackends) {
      if (!backend.isCpu) {
        return backend;
      }
    }
    return AcceleratorBackend.cpu;
  }

  Map<String, Object?> toJson() => {
        'os_family': osFamily,
        'arch': arch,
        'logical_cpu_cores': logicalCpuCores,
        'total_memory_bytes': totalMemoryBytes,
        'available_memory_bytes': availableMemoryBytes,
        'detected_backends': detectedBackends.map((b) => b.wireValue).toList(),
        'cpu_features': cpuFeatures.toJson(),
        'gpus': gpus.map((g) => g.toJson()).toList(),
      };

  factory HardwareProfile.fromJson(Map<String, Object?> json) {
    final backendsRaw = (json['detected_backends'] as List<Object?>?) ?? [];
    final gpusRaw = (json['gpus'] as List<Object?>?) ?? [];
    final cpuFeaturesRaw = json['cpu_features'] as Map<String, Object?>?;

    return HardwareProfile(
      osFamily: json['os_family'] as String? ?? 'unknown',
      arch: json['arch'] as String? ?? 'unknown',
      logicalCpuCores: json['logical_cpu_cores'] as int? ?? 1,
      totalMemoryBytes: json['total_memory_bytes'] as int? ?? 0,
      availableMemoryBytes: json['available_memory_bytes'] as int? ?? 0,
      detectedBackends: backendsRaw
          .map((item) => AcceleratorBackend.fromWire(item as String))
          .toList(),
      cpuFeatures: cpuFeaturesRaw != null
          ? CpuFeatures.fromJson(cpuFeaturesRaw)
          : const CpuFeatures(),
      gpus: gpusRaw
          .map((item) => GpuDevice.fromJson(item as Map<String, Object?>))
          .toList(),
    );
  }
}

class CloudConsentToken {
  const CloudConsentToken({
    required this.tokenId,
    required this.modality,
    required this.targetEndpoint,
    required this.maxPayloadBytes,
    required this.issuedAtEpochSecs,
    required this.expiresAtEpochSecs,
    required this.userAcknowledged,
  });

  final String tokenId;
  final Modality modality;
  final String targetEndpoint;
  final int maxPayloadBytes;
  final int issuedAtEpochSecs;
  final int expiresAtEpochSecs;
  final bool userAcknowledged;

  bool isValidFor({
    required Modality requestedModality,
    required String endpoint,
    required int payloadBytes,
    required int currentEpochSecs,
  }) {
    return userAcknowledged &&
        tokenId.trim().isNotEmpty &&
        targetEndpoint.startsWith('https://') &&
        modality == requestedModality &&
        targetEndpoint == endpoint &&
        payloadBytes <= maxPayloadBytes &&
        currentEpochSecs < expiresAtEpochSecs &&
        expiresAtEpochSecs > issuedAtEpochSecs;
  }

  Map<String, Object?> toJson() => {
        'token_id': tokenId,
        'modality': modality.wireValue,
        'target_endpoint': targetEndpoint,
        'max_payload_bytes': maxPayloadBytes,
        'issued_at_epoch_secs': issuedAtEpochSecs,
        'expires_at_epoch_secs': expiresAtEpochSecs,
        'user_acknowledged': userAcknowledged,
      };

  factory CloudConsentToken.fromJson(Map<String, Object?> json) {
    return CloudConsentToken(
      tokenId: json['token_id'] as String,
      modality: Modality.fromWire(json['modality'] as String),
      targetEndpoint: json['target_endpoint'] as String,
      maxPayloadBytes: json['max_payload_bytes'] as int,
      issuedAtEpochSecs: json['issued_at_epoch_secs'] as int,
      expiresAtEpochSecs: json['expires_at_epoch_secs'] as int,
      userAcknowledged: json['user_acknowledged'] as bool,
    );
  }
}

class AdapterHealth {
  const AdapterHealth({
    required this.adapterId,
    required this.modality,
    required this.ready,
    required this.activeBackend,
    required this.statusMessage,
  });

  final String adapterId;
  final Modality modality;
  final bool ready;
  final AcceleratorBackend activeBackend;
  final String statusMessage;

  Map<String, Object?> toJson() => {
        'adapter_id': adapterId,
        'modality': modality.wireValue,
        'ready': ready,
        'active_backend': activeBackend.wireValue,
        'status_message': statusMessage,
      };

  factory AdapterHealth.fromJson(Map<String, Object?> json) {
    return AdapterHealth(
      adapterId: json['adapter_id'] as String,
      modality: Modality.fromWire(json['modality'] as String),
      ready: json['ready'] as bool,
      activeBackend: AcceleratorBackend.fromWire(
        json['active_backend'] as String,
      ),
      statusMessage: json['status_message'] as String,
    );
  }
}

/// Envelopes typed asynchronous system events emitted by the core or server.
class EventEnvelope {
  const EventEnvelope({
    required this.sequence,
    required this.timestampUtc,
    required this.eventType,
    required this.payload,
  });

  final int sequence;
  final String timestampUtc;
  final String eventType;
  final Map<String, dynamic> payload;

  Map<String, dynamic> toJson() => {
        'sequence': sequence,
        'timestamp_utc': timestampUtc,
        'event_type': eventType,
        'payload': payload,
      };

  factory EventEnvelope.fromJson(Map<String, dynamic> json) {
    return EventEnvelope(
      sequence: json['sequence'] as int? ?? 0,
      timestampUtc: json['timestamp_utc'] as String? ?? '',
      eventType: json['event_type'] as String? ?? 'diagnostic_alert',
      payload: (json['payload'] as Map<String, dynamic>?) ?? {},
    );
  }
}

/// Persistent application configuration settings.
class AppSettings {
  const AppSettings({
    this.localOnlyMode = true,
    this.forceCpuOnly = false,
    this.activeWorkspaceId = 'default',
    this.ipcServerUrl = 'http://127.0.0.1:4140',
    this.sessionNonce = '',
  });

  final bool localOnlyMode;
  final bool forceCpuOnly;
  final String activeWorkspaceId;
  final String ipcServerUrl;
  final String sessionNonce;

  AppSettings copyWith({
    bool? localOnlyMode,
    bool? forceCpuOnly,
    String? activeWorkspaceId,
    String? ipcServerUrl,
    String? sessionNonce,
  }) {
    return AppSettings(
      localOnlyMode: localOnlyMode ?? this.localOnlyMode,
      forceCpuOnly: forceCpuOnly ?? this.forceCpuOnly,
      activeWorkspaceId: activeWorkspaceId ?? this.activeWorkspaceId,
      ipcServerUrl: ipcServerUrl ?? this.ipcServerUrl,
      sessionNonce: sessionNonce ?? this.sessionNonce,
    );
  }

  Map<String, dynamic> toJson() => {
        'local_only_mode': localOnlyMode,
        'force_cpu_only': forceCpuOnly,
        'active_workspace_id': activeWorkspaceId,
        'ipc_server_url': ipcServerUrl,
        'session_nonce': sessionNonce,
      };

  factory AppSettings.fromJson(Map<String, dynamic> json) {
    return AppSettings(
      localOnlyMode: json['local_only_mode'] as bool? ?? true,
      forceCpuOnly: json['force_cpu_only'] as bool? ?? false,
      activeWorkspaceId: json['active_workspace_id'] as String? ?? 'default',
      ipcServerUrl: json['ipc_server_url'] as String? ?? 'http://127.0.0.1:4140',
      sessionNonce: json['session_nonce'] as String? ?? '',
    );
  }
}

enum DownloadStatus {
  pending('pending', 'Pending'),
  downloading('downloading', 'Downloading'),
  verifyingSha256('verifying_sha256', 'Verifying SHA-256'),
  verified('verified', 'Verified'),
  failed('failed', 'Failed'),
  paused('paused', 'Paused');

  const DownloadStatus(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static DownloadStatus fromWire(String value) {
    for (final status in DownloadStatus.values) {
      if (status.wireValue == value) {
        return status;
      }
    }
    return DownloadStatus.pending;
  }
}

class DownloadProgressItem {
  const DownloadProgressItem({
    required this.modelId,
    required this.bytesDownloaded,
    required this.totalBytes,
    required this.percentage,
    required this.status,
    this.errorMessage,
  });

  final String modelId;
  final int bytesDownloaded;
  final int totalBytes;
  final double percentage;
  final DownloadStatus status;
  final String? errorMessage;

  DownloadProgressItem copyWith({
    int? bytesDownloaded,
    int? totalBytes,
    double? percentage,
    DownloadStatus? status,
    String? errorMessage,
  }) {
    return DownloadProgressItem(
      modelId: modelId,
      bytesDownloaded: bytesDownloaded ?? this.bytesDownloaded,
      totalBytes: totalBytes ?? this.totalBytes,
      percentage: percentage ?? this.percentage,
      status: status ?? this.status,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }

  Map<String, dynamic> toJson() => {
        'model_id': modelId,
        'bytes_downloaded': bytesDownloaded,
        'total_bytes': totalBytes,
        'percentage': percentage,
        'status': status.wireValue,
        'error_message': errorMessage,
      };

  factory DownloadProgressItem.fromJson(Map<String, dynamic> json) {
    return DownloadProgressItem(
      modelId: json['model_id'] as String? ?? '',
      bytesDownloaded: json['bytes_downloaded'] as int? ?? 0,
      totalBytes: json['total_bytes'] as int? ?? 0,
      percentage: (json['percentage'] as num?)?.toDouble() ?? 0.0,
      status: DownloadStatus.fromWire(json['status'] as String? ?? 'pending'),
      errorMessage: json['error_message'] as String?,
    );
  }
}

class ModelManifest {
  const ModelManifest({
    required this.modelId,
    required this.name,
    required this.modality,
    required this.version,
    required this.description,
    required this.sourceUrl,
    required this.expectedSha256,
    required this.expectedSizeBytes,
    required this.format,
    this.parameterCount,
    this.quantization,
    required this.minRamBytes,
    required this.recommendedBackend,
  });

  final String modelId;
  final String name;
  final Modality modality;
  final String version;
  final String description;
  final String sourceUrl;
  final String expectedSha256;
  final int expectedSizeBytes;
  final String format;
  final String? parameterCount;
  final String? quantization;
  final int minRamBytes;
  final AcceleratorBackend recommendedBackend;

  String get shortSha256 {
    if (expectedSha256.length <= 12) return expectedSha256;
    return '${expectedSha256.substring(0, 8)}...${expectedSha256.substring(expectedSha256.length - 4)}';
  }

  String get formattedSize {
    if (expectedSizeBytes >= 1024 * 1024 * 1024) {
      return '${(expectedSizeBytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
    }
    return '${(expectedSizeBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  Map<String, dynamic> toJson() => {
        'model_id': modelId,
        'name': name,
        'modality': modality.wireValue,
        'version': version,
        'description': description,
        'source_url': sourceUrl,
        'expected_sha256': expectedSha256,
        'expected_size_bytes': expectedSizeBytes,
        'format': format,
        'parameter_count': parameterCount,
        'quantization': quantization,
        'min_ram_bytes': minRamBytes,
        'recommended_backend': recommendedBackend.wireValue,
      };

  factory ModelManifest.fromJson(Map<String, dynamic> json) {
    return ModelManifest(
      modelId: json['model_id'] as String,
      name: json['name'] as String,
      modality: Modality.fromWire(json['modality'] as String),
      version: json['version'] as String,
      description: json['description'] as String,
      sourceUrl: json['source_url'] as String,
      expectedSha256: json['expected_sha256'] as String,
      expectedSizeBytes: json['expected_size_bytes'] as int,
      format: json['format'] as String,
      parameterCount: json['parameter_count'] as String?,
      quantization: json['quantization'] as String?,
      minRamBytes: json['min_ram_bytes'] as int? ?? 1024 * 1024 * 1024,
      recommendedBackend: AcceleratorBackend.fromWire(
        json['recommended_backend'] as String? ?? 'cpu',
      ),
    );
  }
}

class LocalModelImportRequest {
  const LocalModelImportRequest({
    required this.sourcePath,
    required this.modelId,
    required this.name,
    required this.modality,
    required this.version,
    required this.description,
    required this.format,
    this.parameterCount,
    this.quantization,
    required this.minRamBytes,
    required this.recommendedBackend,
    this.expectedSha256,
    this.expectedSizeBytes,
    this.overwrite = false,
  });

  final String sourcePath;
  final String modelId;
  final String name;
  final Modality modality;
  final String version;
  final String description;
  final String format;
  final String? parameterCount;
  final String? quantization;
  final int minRamBytes;
  final AcceleratorBackend recommendedBackend;
  final String? expectedSha256;
  final int? expectedSizeBytes;
  final bool overwrite;

  Map<String, dynamic> toJson() => {
        'source_path': sourcePath,
        'model_id': modelId,
        'name': name,
        'modality': modality.wireValue,
        'version': version,
        'description': description,
        'format': format,
        'parameter_count': parameterCount,
        'quantization': quantization,
        'min_ram_bytes': minRamBytes,
        'recommended_backend': recommendedBackend.wireValue,
        'expected_sha256': expectedSha256,
        'expected_size_bytes': expectedSizeBytes,
        'overwrite': overwrite,
      };

  factory LocalModelImportRequest.fromJson(Map<String, dynamic> json) {
    return LocalModelImportRequest(
      sourcePath: json['source_path'] as String,
      modelId: json['model_id'] as String,
      name: json['name'] as String,
      modality: Modality.fromWire(json['modality'] as String),
      version: json['version'] as String,
      description: json['description'] as String,
      format: json['format'] as String,
      parameterCount: json['parameter_count'] as String?,
      quantization: json['quantization'] as String?,
      minRamBytes: json['min_ram_bytes'] as int? ?? 1024 * 1024 * 1024,
      recommendedBackend: AcceleratorBackend.fromWire(
        json['recommended_backend'] as String? ?? 'cpu',
      ),
      expectedSha256: json['expected_sha256'] as String?,
      expectedSizeBytes: json['expected_size_bytes'] as int?,
      overwrite: json['overwrite'] as bool? ?? false,
    );
  }
}

enum ChatMessageRole {
  system('system', 'System'),
  user('user', 'User'),
  assistant('assistant', 'Assistant');

  const ChatMessageRole(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static ChatMessageRole fromWire(String value) {
    for (final r in ChatMessageRole.values) {
      if (r.wireValue == value.toLowerCase()) return r;
    }
    return ChatMessageRole.user;
  }
}

class ChatMessage {
  const ChatMessage({
    required this.id,
    required this.sessionId,
    this.parentId,
    required this.role,
    required this.content,
    this.tokenCount = 0,
    this.modelId,
    required this.createdAtEpochSecs,
    required this.updatedAtEpochSecs,
    this.isPinned = false,
  });

  final String id;
  final String sessionId;
  final String? parentId;
  final ChatMessageRole role;
  final String content;
  final int tokenCount;
  final String? modelId;
  final int createdAtEpochSecs;
  final int updatedAtEpochSecs;
  final bool isPinned;

  ChatMessage copyWith({
    String? content,
    int? tokenCount,
    int? updatedAtEpochSecs,
    bool? isPinned,
  }) {
    return ChatMessage(
      id: id,
      sessionId: sessionId,
      parentId: parentId,
      role: role,
      content: content ?? this.content,
      tokenCount: tokenCount ?? this.tokenCount,
      modelId: modelId,
      createdAtEpochSecs: createdAtEpochSecs,
      updatedAtEpochSecs: updatedAtEpochSecs ?? this.updatedAtEpochSecs,
      isPinned: isPinned ?? this.isPinned,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'session_id': sessionId,
        'parent_id': parentId,
        'role': role.wireValue,
        'content': content,
        'token_count': tokenCount,
        'model_id': modelId,
        'created_at_epoch_secs': createdAtEpochSecs,
        'updated_at_epoch_secs': updatedAtEpochSecs,
        'is_pinned': isPinned,
      };

  factory ChatMessage.fromJson(Map<String, dynamic> json) => ChatMessage(
        id: json['id'] as String,
        sessionId: json['session_id'] as String,
        parentId: json['parent_id'] as String?,
        role: ChatMessageRole.fromWire(json['role'] as String? ?? 'user'),
        content: json['content'] as String,
        tokenCount: json['token_count'] as int? ?? 0,
        modelId: json['model_id'] as String?,
        createdAtEpochSecs: json['created_at_epoch_secs'] as int,
        updatedAtEpochSecs: json['updated_at_epoch_secs'] as int,
        isPinned: json['is_pinned'] as bool? ?? false,
      );
}

class ChatSession {
  const ChatSession({
    required this.id,
    required this.title,
    required this.createdAtEpochSecs,
    required this.updatedAtEpochSecs,
  });

  final String id;
  final String title;
  final int createdAtEpochSecs;
  final int updatedAtEpochSecs;

  ChatSession copyWith({
    String? title,
    int? updatedAtEpochSecs,
  }) {
    return ChatSession(
      id: id,
      title: title ?? this.title,
      createdAtEpochSecs: createdAtEpochSecs,
      updatedAtEpochSecs: updatedAtEpochSecs ?? this.updatedAtEpochSecs,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'title': title,
        'created_at_epoch_secs': createdAtEpochSecs,
        'updated_at_epoch_secs': updatedAtEpochSecs,
      };

  factory ChatSession.fromJson(Map<String, dynamic> json) => ChatSession(
        id: json['id'] as String,
        title: json['title'] as String,
        createdAtEpochSecs: json['created_at_epoch_secs'] as int,
        updatedAtEpochSecs: json['updated_at_epoch_secs'] as int,
      );
}


