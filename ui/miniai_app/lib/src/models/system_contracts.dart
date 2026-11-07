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
    this.citations = const [],
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
  final List<String> citations;

  ChatMessage copyWith({
    String? content,
    int? tokenCount,
    int? updatedAtEpochSecs,
    bool? isPinned,
    List<String>? citations,
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
      citations: citations ?? this.citations,
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
        'citations': citations,
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
        citations: (json['citations'] as List<dynamic>?)
                ?.map((e) => e as String)
                .toList() ??
            const [],
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



enum TrainingMethod {
  lora('lora', 'LoRA (Low-Rank Adaptation)'),
  qlora('qlora', 'QLoRA (Quantized LoRA)'),
  distillation('distillation', 'Knowledge Distillation'),
  full('full', 'Full Parameter Tuning');

  const TrainingMethod(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static TrainingMethod fromWire(String value) {
    for (final m in TrainingMethod.values) {
      if (m.wireValue == value) return m;
    }
    return TrainingMethod.lora;
  }
}

enum TrainingJobStatus {
  idle('idle', 'Idle'),
  preparing('preparing', 'Preparing Dataset'),
  training('training', 'Training in Progress'),
  evaluating('evaluating', 'Evaluating Checkpoint'),
  completed('completed', 'Completed & Certified'),
  failed('failed', 'Failed'),
  cancelled('cancelled', 'Cancelled');

  const TrainingJobStatus(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static TrainingJobStatus fromWire(String value) {
    for (final s in TrainingJobStatus.values) {
      if (s.wireValue == value) return s;
    }
    return TrainingJobStatus.idle;
  }
}

class TrainingHyperparameters {
  const TrainingHyperparameters({
    this.rank = 8,
    this.loraAlpha = 16.0,
    this.loraDropout = 0.05,
    this.learningRate = 0.0001,
    this.batchSize = 4,
    this.epochs = 3,
    this.gradientAccumulationSteps = 1,
    this.seed = 42,
    this.targetModules = const ['q_proj', 'v_proj'],
  });

  final int rank;
  final double loraAlpha;
  final double loraDropout;
  final double learningRate;
  final int batchSize;
  final int epochs;
  final int gradientAccumulationSteps;
  final int seed;
  final List<String> targetModules;

  TrainingHyperparameters copyWith({
    int? rank,
    double? loraAlpha,
    double? loraDropout,
    double? learningRate,
    int? batchSize,
    int? epochs,
    int? gradientAccumulationSteps,
    int? seed,
    List<String>? targetModules,
  }) {
    return TrainingHyperparameters(
      rank: rank ?? this.rank,
      loraAlpha: loraAlpha ?? this.loraAlpha,
      loraDropout: loraDropout ?? this.loraDropout,
      learningRate: learningRate ?? this.learningRate,
      batchSize: batchSize ?? this.batchSize,
      epochs: epochs ?? this.epochs,
      gradientAccumulationSteps:
          gradientAccumulationSteps ?? this.gradientAccumulationSteps,
      seed: seed ?? this.seed,
      targetModules: targetModules ?? this.targetModules,
    );
  }

  Map<String, dynamic> toJson() => {
        'rank': rank,
        'lora_alpha': loraAlpha,
        'lora_dropout': loraDropout,
        'learning_rate': learningRate,
        'batch_size': batchSize,
        'epochs': epochs,
        'gradient_accumulation_steps': gradientAccumulationSteps,
        'seed': seed,
        'target_modules': targetModules,
      };

  factory TrainingHyperparameters.fromJson(Map<String, dynamic> json) =>
      TrainingHyperparameters(
        rank: json['rank'] as int? ?? 8,
        loraAlpha: (json['lora_alpha'] as num?)?.toDouble() ?? 16.0,
        loraDropout: (json['lora_dropout'] as num?)?.toDouble() ?? 0.05,
        learningRate: (json['learning_rate'] as num?)?.toDouble() ?? 0.0001,
        batchSize: json['batch_size'] as int? ?? 4,
        epochs: json['epochs'] as int? ?? 3,
        gradientAccumulationSteps:
            json['gradient_accumulation_steps'] as int? ?? 1,
        seed: json['seed'] as int? ?? 42,
        targetModules: (json['target_modules'] as List<dynamic>?)
                ?.map((e) => e.toString())
                .toList() ??
            const ['q_proj', 'v_proj'],
      );
}

class DatasetSplitInfo {
  const DatasetSplitInfo({
    required this.name,
    required this.totalRecords,
    required this.trainRecords,
    required this.valRecords,
    required this.testRecords,
    required this.sha256Digest,
  });

  final String name;
  final int totalRecords;
  final int trainRecords;
  final int valRecords;
  final int testRecords;
  final String sha256Digest;

  Map<String, dynamic> toJson() => {
        'name': name,
        'total_records': totalRecords,
        'train_records': trainRecords,
        'val_records': valRecords,
        'test_records': testRecords,
        'sha256_digest': sha256Digest,
      };

  factory DatasetSplitInfo.fromJson(Map<String, dynamic> json) =>
      DatasetSplitInfo(
        name: json['name'] as String,
        totalRecords: json['total_records'] as int,
        trainRecords: json['train_records'] as int,
        valRecords: json['val_records'] as int,
        testRecords: json['test_records'] as int,
        sha256Digest: json['sha256_digest'] as String,
      );
}

class TrainingJob {
  const TrainingJob({
    required this.id,
    required this.baseModelId,
    required this.datasetName,
    required this.method,
    required this.status,
    required this.hyperparameters,
    this.currentEpoch = 0,
    this.currentStep = 0,
    this.totalSteps = 100,
    this.currentLoss = 0.0,
    this.valLoss = 0.0,
    this.perplexity = 1.0,
    this.lossHistory = const [],
    this.errorMessage,
    this.outputAdapterPath,
    this.adapterSha256,
  });

  final String id;
  final String baseModelId;
  final String datasetName;
  final TrainingMethod method;
  final TrainingJobStatus status;
  final TrainingHyperparameters hyperparameters;
  final int currentEpoch;
  final int currentStep;
  final int totalSteps;
  final double currentLoss;
  final double valLoss;
  final double perplexity;
  final List<double> lossHistory;
  final String? errorMessage;
  final String? outputAdapterPath;
  final String? adapterSha256;

  double get progress =>
      totalSteps > 0 ? (currentStep / totalSteps).clamp(0.0, 1.0) : 0.0;

  TrainingJob copyWith({
    TrainingJobStatus? status,
    int? currentEpoch,
    int? currentStep,
    int? totalSteps,
    double? currentLoss,
    double? valLoss,
    double? perplexity,
    List<double>? lossHistory,
    String? errorMessage,
    String? outputAdapterPath,
    String? adapterSha256,
  }) {
    return TrainingJob(
      id: id,
      baseModelId: baseModelId,
      datasetName: datasetName,
      method: method,
      status: status ?? this.status,
      hyperparameters: hyperparameters,
      currentEpoch: currentEpoch ?? this.currentEpoch,
      currentStep: currentStep ?? this.currentStep,
      totalSteps: totalSteps ?? this.totalSteps,
      currentLoss: currentLoss ?? this.currentLoss,
      valLoss: valLoss ?? this.valLoss,
      perplexity: perplexity ?? this.perplexity,
      lossHistory: lossHistory ?? this.lossHistory,
      errorMessage: errorMessage ?? this.errorMessage,
      outputAdapterPath: outputAdapterPath ?? this.outputAdapterPath,
      adapterSha256: adapterSha256 ?? this.adapterSha256,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'base_model_id': baseModelId,
        'dataset_name': datasetName,
        'method': method.wireValue,
        'status': status.wireValue,
        'hyperparameters': hyperparameters.toJson(),
        'current_epoch': currentEpoch,
        'current_step': currentStep,
        'total_steps': totalSteps,
        'current_loss': currentLoss,
        'val_loss': valLoss,
        'perplexity': perplexity,
        'loss_history': lossHistory,
        'error_message': errorMessage,
        'output_adapter_path': outputAdapterPath,
        'adapter_sha256': adapterSha256,
      };

  factory TrainingJob.fromJson(Map<String, dynamic> json) => TrainingJob(
        id: json['id'] as String,
        baseModelId: json['base_model_id'] as String,
        datasetName: json['dataset_name'] as String,
        method: TrainingMethod.fromWire(json['method'] as String? ?? 'lora'),
        status: TrainingJobStatus.fromWire(json['status'] as String? ?? 'idle'),
        hyperparameters: TrainingHyperparameters.fromJson(
          json['hyperparameters'] as Map<String, dynamic>? ?? {},
        ),
        currentEpoch: json['current_epoch'] as int? ?? 0,
        currentStep: json['current_step'] as int? ?? 0,
        totalSteps: json['total_steps'] as int? ?? 100,
        currentLoss: (json['current_loss'] as num?)?.toDouble() ?? 0.0,
        valLoss: (json['val_loss'] as num?)?.toDouble() ?? 0.0,
        perplexity: (json['perplexity'] as num?)?.toDouble() ?? 1.0,
        lossHistory: (json['loss_history'] as List<dynamic>?)
                ?.map((e) => (e as num).toDouble())
                .toList() ??
            const [],
        errorMessage: json['error_message'] as String?,
        outputAdapterPath: json['output_adapter_path'] as String?,
        adapterSha256: json['adapter_sha256'] as String?,
      );
}

// -----------------------------------------------------------------------------
// Stage 5: RAG & Knowledge Base Contracts
// -----------------------------------------------------------------------------

enum KnowledgeDocType {
  markdown('markdown', 'Markdown'),
  code('code', 'Source Code'),
  plainText('plain_text', 'Plain Text'),
  pdfText('pdf_text', 'PDF Text');

  const KnowledgeDocType(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static KnowledgeDocType fromWire(String value) {
    for (final t in KnowledgeDocType.values) {
      if (t.wireValue == value.toLowerCase()) return t;
    }
    return KnowledgeDocType.plainText;
  }
}

class KnowledgeDocument {
  const KnowledgeDocument({
    required this.filePath,
    required this.fileSha256,
    required this.sizeBytes,
    required this.lineCount,
    required this.docType,
    required this.lastModifiedEpochSecs,
  });

  final String filePath;
  final String fileSha256;
  final int sizeBytes;
  final int lineCount;
  final KnowledgeDocType docType;
  final int lastModifiedEpochSecs;

  String get formattedSize {
    if (sizeBytes >= 1024 * 1024) {
      return '${(sizeBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    if (sizeBytes >= 1024) {
      return '${(sizeBytes / 1024).toStringAsFixed(1)} KB';
    }
    return '$sizeBytes B';
  }

  Map<String, dynamic> toJson() => {
        'file_path': filePath,
        'file_sha256': fileSha256,
        'size_bytes': sizeBytes,
        'line_count': lineCount,
        'doc_type': docType.wireValue,
        'last_modified_epoch_secs': lastModifiedEpochSecs,
      };

  factory KnowledgeDocument.fromJson(Map<String, dynamic> json) =>
      KnowledgeDocument(
        filePath: json['file_path'] as String,
        fileSha256: json['file_sha256'] as String,
        sizeBytes: json['size_bytes'] as int? ?? 0,
        lineCount: json['line_count'] as int? ?? 0,
        docType: KnowledgeDocType.fromWire(json['doc_type'] as String? ?? 'plain_text'),
        lastModifiedEpochSecs: json['last_modified_epoch_secs'] as int? ?? 0,
      );
}

class KnowledgeChunk {
  const KnowledgeChunk({
    required this.chunkId,
    required this.filePath,
    required this.lineStart,
    required this.lineEnd,
    required this.text,
    required this.citationTag,
    required this.tokenEstimate,
    required this.docType,
  });

  final String chunkId;
  final String filePath;
  final int lineStart;
  final int lineEnd;
  final String text;
  final String citationTag;
  final int tokenEstimate;
  final KnowledgeDocType docType;

  Map<String, dynamic> toJson() => {
        'chunk_id': chunkId,
        'file_path': filePath,
        'line_start': lineStart,
        'line_end': lineEnd,
        'text': text,
        'citation_tag': citationTag,
        'token_estimate': tokenEstimate,
        'doc_type': docType.wireValue,
      };

  factory KnowledgeChunk.fromJson(Map<String, dynamic> json) => KnowledgeChunk(
        chunkId: json['chunk_id'] as String,
        filePath: json['file_path'] as String,
        lineStart: json['line_start'] as int? ?? 1,
        lineEnd: json['line_end'] as int? ?? 1,
        text: json['text'] as String? ?? '',
        citationTag: json['citation_tag'] as String? ?? '',
        tokenEstimate: json['token_estimate'] as int? ?? 0,
        docType: KnowledgeDocType.fromWire(json['doc_type'] as String? ?? 'plain_text'),
      );
}

class RetrievalResultItem {
  const RetrievalResultItem({
    required this.chunk,
    required this.rrfScore,
    required this.denseScore,
    required this.lexicalScore,
    required this.citationTag,
    required this.excerpt,
  });

  final KnowledgeChunk chunk;
  final double rrfScore;
  final double denseScore;
  final double lexicalScore;
  final String citationTag;
  final String excerpt;

  Map<String, dynamic> toJson() => {
        'chunk': chunk.toJson(),
        'rrf_score': rrfScore,
        'dense_score': denseScore,
        'lexical_score': lexicalScore,
        'citation_tag': citationTag,
        'excerpt': excerpt,
      };

  factory RetrievalResultItem.fromJson(Map<String, dynamic> json) =>
      RetrievalResultItem(
        chunk: KnowledgeChunk.fromJson(json['chunk'] as Map<String, dynamic>),
        rrfScore: (json['rrf_score'] as num?)?.toDouble() ?? 0.0,
        denseScore: (json['dense_score'] as num?)?.toDouble() ?? 0.0,
        lexicalScore: (json['lexical_score'] as num?)?.toDouble() ?? 0.0,
        citationTag: json['citation_tag'] as String? ?? '',
        excerpt: json['excerpt'] as String? ?? '',
      );
}

enum ToolCapabilityContract {
  workspaceRead('workspace_read', 'Workspace Read'),
  compute('compute', 'Local Compute'),
  codeSearch('code_search', 'Code Search'),
  network('network', 'Network Outbound');

  const ToolCapabilityContract(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static ToolCapabilityContract fromWire(String value) {
    for (final c in ToolCapabilityContract.values) {
      if (c.wireValue == value.toLowerCase()) return c;
    }
    return ToolCapabilityContract.compute;
  }
}

class ToolDescriptorContract {
  const ToolDescriptorContract({
    required this.name,
    required this.description,
    required this.requiredCapability,
  });

  final String name;
  final String description;
  final ToolCapabilityContract requiredCapability;

  Map<String, dynamic> toJson() => {
        'name': name,
        'description': description,
        'required_capability': requiredCapability.wireValue,
      };

  factory ToolDescriptorContract.fromJson(Map<String, dynamic> json) =>
      ToolDescriptorContract(
        name: json['name'] as String,
        description: json['description'] as String,
        requiredCapability: ToolCapabilityContract.fromWire(
          json['required_capability'] as String? ?? 'compute',
        ),
      );
}

class KnowledgeBaseStats {
  const KnowledgeBaseStats({
    required this.filesIndexed,
    required this.chunksIndexed,
    required this.totalCharsIndexed,
    required this.lastIndexedAtEpochSecs,
  });

  final int filesIndexed;
  final int chunksIndexed;
  final int totalCharsIndexed;
  final int lastIndexedAtEpochSecs;

  Map<String, dynamic> toJson() => {
        'files_indexed': filesIndexed,
        'chunks_indexed': chunksIndexed,
        'total_chars_indexed': totalCharsIndexed,
        'last_indexed_at_epoch_secs': lastIndexedAtEpochSecs,
      };

  factory KnowledgeBaseStats.fromJson(Map<String, dynamic> json) =>
      KnowledgeBaseStats(
        filesIndexed: json['files_indexed'] as int? ?? 0,
        chunksIndexed: json['chunks_indexed'] as int? ?? 0,
        totalCharsIndexed: json['total_chars_indexed'] as int? ?? 0,
        lastIndexedAtEpochSecs: json['last_indexed_at_epoch_secs'] as int? ?? 0,
      );
}

/// OCR detected bounding box contract.
class OcrBoxContract {
  const OcrBoxContract({
    required this.text,
    required this.confidence,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
  });

  final String text;
  final double confidence;
  final double x;
  final double y;
  final double width;
  final double height;

  Map<String, dynamic> toJson() => {
        'text': text,
        'confidence': confidence,
        'x': x,
        'y': y,
        'width': width,
        'height': height,
      };

  factory OcrBoxContract.fromJson(Map<String, dynamic> json) => OcrBoxContract(
        text: json['text'] as String? ?? '',
        confidence: (json['confidence'] as num?)?.toDouble() ?? 0.0,
        x: (json['x'] as num?)?.toDouble() ?? 0.0,
        y: (json['y'] as num?)?.toDouble() ?? 0.0,
        width: (json['width'] as num?)?.toDouble() ?? 0.0,
        height: (json['height'] as num?)?.toDouble() ?? 0.0,
      );
}

/// Visual understanding analysis result contract.
class VisionAnalysisResultContract {
  const VisionAnalysisResultContract({
    required this.imageSha256,
    required this.width,
    required this.height,
    required this.format,
    required this.description,
    required this.tags,
    required this.ocrBoxes,
    this.qaAnswer,
  });

  final String imageSha256;
  final int width;
  final int height;
  final String format;
  final String description;
  final List<String> tags;
  final List<OcrBoxContract> ocrBoxes;
  final String? qaAnswer;

  Map<String, dynamic> toJson() => {
        'image_sha256': imageSha256,
        'width': width,
        'height': height,
        'format': format,
        'description': description,
        'tags': tags,
        'ocr_boxes': ocrBoxes.map((b) => b.toJson()).toList(),
        if (qaAnswer != null) 'qa_answer': qaAnswer,
      };

  factory VisionAnalysisResultContract.fromJson(Map<String, dynamic> json) =>
      VisionAnalysisResultContract(
        imageSha256: json['image_sha256'] as String? ?? '',
        width: json['width'] as int? ?? 0,
        height: json['height'] as int? ?? 0,
        format: json['format'] as String? ?? 'UNKNOWN',
        description: json['description'] as String? ?? '',
        tags: (json['tags'] as List<dynamic>?)?.map((t) => t.toString()).toList() ?? const [],
        ocrBoxes: (json['ocr_boxes'] as List<dynamic>?)
                ?.map((b) => OcrBoxContract.fromJson(b as Map<String, dynamic>))
                .toList() ??
            const [],
        qaAnswer: json['qa_answer'] as String?,
      );
}

/// Image generation mode contract.
enum GenerationModeContract {
  textToImage('text_to_image', 'Text-to-Image'),
  imageToImage('image_to_image', 'Image-to-Image'),
  inpainting('inpainting', 'Inpainting Canvas');

  const GenerationModeContract(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static GenerationModeContract fromWire(String value) {
    for (final mode in GenerationModeContract.values) {
      if (mode.wireValue == value.toLowerCase()) return mode;
    }
    return GenerationModeContract.textToImage;
  }
}

/// Image generation task parameters.
class ImageGenTaskConfig {
  const ImageGenTaskConfig({
    required this.prompt,
    this.negativePrompt = '',
    this.width = 256,
    this.height = 256,
    this.steps = 20,
    this.guidanceScale = 7.5,
    this.seed = 42,
    this.strength = 0.75,
    this.mode = GenerationModeContract.textToImage,
  });

  final String prompt;
  final String negativePrompt;
  final int width;
  final int height;
  final int steps;
  final double guidanceScale;
  final int seed;
  final double strength;
  final GenerationModeContract mode;

  bool get isValid =>
      prompt.trim().isNotEmpty &&
      (width % 64 == 0) &&
      (height % 64 == 0) &&
      width >= 64 &&
      width <= 1024 &&
      height >= 64 &&
      height <= 1024 &&
      steps >= 1 &&
      steps <= 100 &&
      guidanceScale >= 1.0 &&
      guidanceScale <= 30.0 &&
      strength >= 0.0 &&
      strength <= 1.0;

  ImageGenTaskConfig copyWith({
    String? prompt,
    String? negativePrompt,
    int? width,
    int? height,
    int? steps,
    double? guidanceScale,
    int? seed,
    double? strength,
    GenerationModeContract? mode,
  }) {
    return ImageGenTaskConfig(
      prompt: prompt ?? this.prompt,
      negativePrompt: negativePrompt ?? this.negativePrompt,
      width: width ?? this.width,
      height: height ?? this.height,
      steps: steps ?? this.steps,
      guidanceScale: guidanceScale ?? this.guidanceScale,
      seed: seed ?? this.seed,
      strength: strength ?? this.strength,
      mode: mode ?? this.mode,
    );
  }

  Map<String, dynamic> toJson() => {
        'prompt': prompt,
        'negative_prompt': negativePrompt,
        'width': width,
        'height': height,
        'steps': steps,
        'guidance_scale': guidanceScale,
        'seed': seed,
        'strength': strength,
        'mode': mode.wireValue,
      };

  factory ImageGenTaskConfig.fromJson(Map<String, dynamic> json) => ImageGenTaskConfig(
        prompt: json['prompt'] as String? ?? '',
        negativePrompt: json['negative_prompt'] as String? ?? '',
        width: json['width'] as int? ?? 256,
        height: json['height'] as int? ?? 256,
        steps: json['steps'] as int? ?? 20,
        guidanceScale: (json['guidance_scale'] as num?)?.toDouble() ?? 7.5,
        seed: json['seed'] as int? ?? 42,
        strength: (json['strength'] as num?)?.toDouble() ?? 0.75,
        mode: GenerationModeContract.fromWire(json['mode'] as String? ?? 'text_to_image'),
      );
}

/// Generated image artifact in local canvas gallery.
class GeneratedImageItem {
  const GeneratedImageItem({
    required this.id,
    required this.prompt,
    required this.negativePrompt,
    required this.mode,
    required this.width,
    required this.height,
    required this.steps,
    required this.guidanceScale,
    required this.seed,
    required this.sha256,
    required this.timestampEpochSecs,
    required this.base64Png,
    required this.provenanceSummary,
    required this.sizeBytes,
  });

  final String id;
  final String prompt;
  final String negativePrompt;
  final GenerationModeContract mode;
  final int width;
  final int height;
  final int steps;
  final double guidanceScale;
  final int seed;
  final String sha256;
  final int timestampEpochSecs;
  final String base64Png;
  final String provenanceSummary;
  final int sizeBytes;

  String get formattedSize {
    if (sizeBytes >= 1024 * 1024) {
      return '${(sizeBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    if (sizeBytes >= 1024) {
      return '${(sizeBytes / 1024).toStringAsFixed(1)} KB';
    }
    return '$sizeBytes B';
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'prompt': prompt,
        'negative_prompt': negativePrompt,
        'mode': mode.wireValue,
        'width': width,
        'height': height,
        'steps': steps,
        'guidance_scale': guidanceScale,
        'seed': seed,
        'sha256': sha256,
        'timestamp_epoch_secs': timestampEpochSecs,
        'base64_png': base64Png,
        'provenance_summary': provenanceSummary,
        'size_bytes': sizeBytes,
      };

  factory GeneratedImageItem.fromJson(Map<String, dynamic> json) => GeneratedImageItem(
        id: json['id'] as String? ?? '',
        prompt: json['prompt'] as String? ?? '',
        negativePrompt: json['negative_prompt'] as String? ?? '',
        mode: GenerationModeContract.fromWire(json['mode'] as String? ?? 'text_to_image'),
        width: json['width'] as int? ?? 256,
        height: json['height'] as int? ?? 256,
        steps: json['steps'] as int? ?? 20,
        guidanceScale: (json['guidance_scale'] as num?)?.toDouble() ?? 7.5,
        seed: json['seed'] as int? ?? 42,
        sha256: json['sha256'] as String? ?? '',
        timestampEpochSecs: json['timestamp_epoch_secs'] as int? ?? 0,
        base64Png: json['base64_png'] as String? ?? '',
        provenanceSummary: json['provenance_summary'] as String? ?? '',
        sizeBytes: json['size_bytes'] as int? ?? 0,
      );
}

// -----------------------------------------------------------------------------
// Stage 7: Speech, Voice & Music Audio Contracts
// -----------------------------------------------------------------------------

class SpeechSegmentContract {
  const SpeechSegmentContract({
    required this.startSecs,
    required this.endSecs,
    required this.text,
    required this.confidence,
  });

  final double startSecs;
  final double endSecs;
  final String text;
  final double confidence;

  Map<String, dynamic> toJson() => {
        'start_secs': startSecs,
        'end_secs': endSecs,
        'text': text,
        'confidence': confidence,
      };

  factory SpeechSegmentContract.fromJson(Map<String, dynamic> json) =>
      SpeechSegmentContract(
        startSecs: (json['start_secs'] as num?)?.toDouble() ?? 0.0,
        endSecs: (json['end_secs'] as num?)?.toDouble() ?? 0.0,
        text: json['text'] as String? ?? '',
        confidence: (json['confidence'] as num?)?.toDouble() ?? 0.0,
      );
}

class SpeechAsrResultContract {
  const SpeechAsrResultContract({
    required this.fullText,
    required this.segments,
    required this.confidence,
    required this.durationSecs,
    required this.sha256,
  });

  final String fullText;
  final List<SpeechSegmentContract> segments;
  final double confidence;
  final double durationSecs;
  final String sha256;

  Map<String, dynamic> toJson() => {
        'full_text': fullText,
        'segments': segments.map((s) => s.toJson()).toList(),
        'confidence': confidence,
        'duration_secs': durationSecs,
        'sha256': sha256,
      };

  factory SpeechAsrResultContract.fromJson(Map<String, dynamic> json) =>
      SpeechAsrResultContract(
        fullText: json['full_text'] as String? ?? '',
        segments: (json['segments'] as List<dynamic>?)
                ?.map((e) =>
                    SpeechSegmentContract.fromJson(e as Map<String, dynamic>))
                .toList() ??
            const [],
        confidence: (json['confidence'] as num?)?.toDouble() ?? 0.0,
        durationSecs: (json['duration_secs'] as num?)?.toDouble() ?? 0.0,
        sha256: json['sha256'] as String? ?? '',
      );
}

enum TtsVoiceProfileContract {
  alloy('alloy', 'Alloy (Neutral & Balanced)'),
  echo('echo', 'Echo (Warm & Resonant)'),
  fable('fable', 'Fable (Expressive Narrative)'),
  onyx('onyx', 'Onyx (Deep Baritone)'),
  nova('nova', 'Nova (Bright & Dynamic)'),
  shimmer('shimmer', 'Shimmer (Clear & Melodic)');

  const TtsVoiceProfileContract(this.wireValue, this.displayName);
  final String wireValue;
  final String displayName;

  static TtsVoiceProfileContract fromWire(String val) {
    final lower = val.toLowerCase();
    for (final v in TtsVoiceProfileContract.values) {
      if (v.wireValue == lower) return v;
    }
    return TtsVoiceProfileContract.alloy;
  }
}

class TtsSynthesisConfigContract {
  const TtsSynthesisConfigContract({
    this.voice = TtsVoiceProfileContract.alloy,
    this.sampleRate = 22050,
    this.speed = 1.0,
    this.pitchSemitones = 0.0,
    this.volumeGain = 1.0,
    this.seed = 42,
  });

  final TtsVoiceProfileContract voice;
  final int sampleRate;
  final double speed;
  final double pitchSemitones;
  final double volumeGain;
  final int? seed;

  Map<String, dynamic> toJson() => {
        'voice': voice.wireValue,
        'sample_rate': sampleRate,
        'speed': speed,
        'pitch_semitones': pitchSemitones,
        'volume_gain': volumeGain,
        if (seed != null) 'seed': seed,
      };

  factory TtsSynthesisConfigContract.fromJson(Map<String, dynamic> json) =>
      TtsSynthesisConfigContract(
        voice: TtsVoiceProfileContract.fromWire(
            json['voice'] as String? ?? 'alloy'),
        sampleRate: json['sample_rate'] as int? ?? 22050,
        speed: (json['speed'] as num?)?.toDouble() ?? 1.0,
        pitchSemitones:
            (json['pitch_semitones'] as num?)?.toDouble() ?? 0.0,
        volumeGain: (json['volume_gain'] as num?)?.toDouble() ?? 1.0,
        seed: json['seed'] as int?,
      );
}

class TtsOutputContract {
  const TtsOutputContract({
    required this.durationSecs,
    required this.sampleRate,
    required this.channels,
    required this.sha256,
    required this.phonemeCount,
    this.wavBytesBase64 = '',
  });

  final double durationSecs;
  final int sampleRate;
  final int channels;
  final String sha256;
  final int phonemeCount;
  final String wavBytesBase64;

  Map<String, dynamic> toJson() => {
        'duration_secs': durationSecs,
        'sample_rate': sampleRate,
        'channels': channels,
        'sha256': sha256,
        'phoneme_count': phonemeCount,
        'wav_bytes_base64': wavBytesBase64,
      };

  factory TtsOutputContract.fromJson(Map<String, dynamic> json) =>
      TtsOutputContract(
        durationSecs: (json['duration_secs'] as num?)?.toDouble() ?? 0.0,
        sampleRate: json['sample_rate'] as int? ?? 22050,
        channels: json['channels'] as int? ?? 1,
        sha256: json['sha256'] as String? ?? '',
        phonemeCount: json['phoneme_count'] as int? ?? 0,
        wavBytesBase64: json['wav_bytes_base64'] as String? ?? '',
      );
}

class VoiceTurnMetricsContract {
  const VoiceTurnMetricsContract({
    required this.asrLatencyMs,
    required this.llmTtftMs,
    required this.llmTotalLatencyMs,
    required this.ttsLatencyMs,
    required this.totalTurnaroundMs,
  });

  final double asrLatencyMs;
  final double llmTtftMs;
  final double llmTotalLatencyMs;
  final double ttsLatencyMs;
  final double totalTurnaroundMs;

  Map<String, dynamic> toJson() => {
        'asr_latency_ms': asrLatencyMs,
        'llm_ttft_ms': llmTtftMs,
        'llm_total_latency_ms': llmTotalLatencyMs,
        'tts_latency_ms': ttsLatencyMs,
        'total_turnaround_ms': totalTurnaroundMs,
      };

  factory VoiceTurnMetricsContract.fromJson(Map<String, dynamic> json) =>
      VoiceTurnMetricsContract(
        asrLatencyMs: (json['asr_latency_ms'] as num?)?.toDouble() ?? 0.0,
        llmTtftMs: (json['llm_ttft_ms'] as num?)?.toDouble() ?? 0.0,
        llmTotalLatencyMs:
            (json['llm_total_latency_ms'] as num?)?.toDouble() ?? 0.0,
        ttsLatencyMs: (json['tts_latency_ms'] as num?)?.toDouble() ?? 0.0,
        totalTurnaroundMs:
            (json['total_turnaround_ms'] as num?)?.toDouble() ?? 0.0,
      );
}

class VoiceTurnResultContract {
  const VoiceTurnResultContract({
    required this.id,
    required this.userTranscription,
    required this.agentResponseText,
    required this.synthesizedAudio,
    required this.metrics,
    required this.timestampEpochSecs,
  });

  final String id;
  final String userTranscription;
  final String agentResponseText;
  final TtsOutputContract synthesizedAudio;
  final VoiceTurnMetricsContract metrics;
  final int timestampEpochSecs;

  Map<String, dynamic> toJson() => {
        'id': id,
        'user_transcription': userTranscription,
        'agent_response_text': agentResponseText,
        'synthesized_audio': synthesizedAudio.toJson(),
        'metrics': metrics.toJson(),
        'timestamp_epoch_secs': timestampEpochSecs,
      };

  factory VoiceTurnResultContract.fromJson(Map<String, dynamic> json) =>
      VoiceTurnResultContract(
        id: json['id'] as String? ?? '',
        userTranscription: json['user_transcription'] as String? ?? '',
        agentResponseText: json['agent_response_text'] as String? ?? '',
        synthesizedAudio: TtsOutputContract.fromJson(
            json['synthesized_audio'] as Map<String, dynamic>? ?? {}),
        metrics: VoiceTurnMetricsContract.fromJson(
            json['metrics'] as Map<String, dynamic>? ?? {}),
        timestampEpochSecs: json['timestamp_epoch_secs'] as int? ?? 0,
      );
}

class MusicGenConfigContract {
  const MusicGenConfigContract({
    this.prompt = 'Uplifting ambient strings in C Major',
    this.durationSecs = 10.0,
    this.tempoBpm = 120,
    this.sampleRate = 44100,
    this.channels = 2,
    this.scaleKey = 'C Major',
    this.seed = 42,
  });

  final String prompt;
  final double durationSecs;
  final int tempoBpm;
  final int sampleRate;
  final int channels;
  final String scaleKey;
  final int? seed;

  Map<String, dynamic> toJson() => {
        'prompt': prompt,
        'duration_secs': durationSecs,
        'tempo_bpm': tempoBpm,
        'sample_rate': sampleRate,
        'channels': channels,
        'scale_key': scaleKey,
        if (seed != null) 'seed': seed,
      };

  factory MusicGenConfigContract.fromJson(Map<String, dynamic> json) =>
      MusicGenConfigContract(
        prompt: json['prompt'] as String? ?? '',
        durationSecs: (json['duration_secs'] as num?)?.toDouble() ?? 10.0,
        tempoBpm: json['tempo_bpm'] as int? ?? 120,
        sampleRate: json['sample_rate'] as int? ?? 44100,
        channels: json['channels'] as int? ?? 2,
        scaleKey: json['scale_key'] as String? ?? 'C Major',
        seed: json['seed'] as int?,
      );
}

class MusicTrackOutputContract {
  const MusicTrackOutputContract({
    required this.id,
    required this.prompt,
    required this.durationSecs,
    required this.sampleRate,
    required this.channels,
    required this.tempoBpm,
    required this.scaleKey,
    required this.sha256,
    required this.numSamplesPerChannel,
    required this.timestampEpochSecs,
    this.wavBytesBase64 = '',
  });

  final String id;
  final String prompt;
  final double durationSecs;
  final int sampleRate;
  final int channels;
  final int tempoBpm;
  final String scaleKey;
  final String sha256;
  final int numSamplesPerChannel;
  final int timestampEpochSecs;
  final String wavBytesBase64;

  Map<String, dynamic> toJson() => {
        'id': id,
        'prompt': prompt,
        'duration_secs': durationSecs,
        'sample_rate': sampleRate,
        'channels': channels,
        'tempo_bpm': tempoBpm,
        'scale_key': scaleKey,
        'sha256': sha256,
        'num_samples_per_channel': numSamplesPerChannel,
        'timestamp_epoch_secs': timestampEpochSecs,
        'wav_bytes_base64': wavBytesBase64,
      };

  factory MusicTrackOutputContract.fromJson(Map<String, dynamic> json) =>
      MusicTrackOutputContract(
        id: json['id'] as String? ?? '',
        prompt: json['prompt'] as String? ?? '',
        durationSecs: (json['duration_secs'] as num?)?.toDouble() ?? 10.0,
        sampleRate: json['sample_rate'] as int? ?? 44100,
        channels: json['channels'] as int? ?? 2,
        tempoBpm: json['tempo_bpm'] as int? ?? 120,
        scaleKey: json['scale_key'] as String? ?? 'C Major',
        sha256: json['sha256'] as String? ?? '',
        numSamplesPerChannel: json['num_samples_per_channel'] as int? ?? 0,
        timestampEpochSecs: json['timestamp_epoch_secs'] as int? ?? 0,
        wavBytesBase64: json['wav_bytes_base64'] as String? ?? '',
      );
}
