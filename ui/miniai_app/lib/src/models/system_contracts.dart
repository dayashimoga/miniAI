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

class HardwareProfile {
  const HardwareProfile({
    required this.osFamily,
    required this.arch,
    required this.logicalCpuCores,
    required this.totalMemoryBytes,
    required this.availableMemoryBytes,
    required this.detectedBackends,
  });

  final String osFamily;
  final String arch;
  final int logicalCpuCores;
  final int totalMemoryBytes;
  final int availableMemoryBytes;
  final List<AcceleratorBackend> detectedBackends;

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
      };

  factory HardwareProfile.fromJson(Map<String, Object?> json) {
    final backendsRaw = json['detected_backends'] as List<Object?>;
    return HardwareProfile(
      osFamily: json['os_family'] as String,
      arch: json['arch'] as String,
      logicalCpuCores: json['logical_cpu_cores'] as int,
      totalMemoryBytes: json['total_memory_bytes'] as int,
      availableMemoryBytes: json['available_memory_bytes'] as int,
      detectedBackends: backendsRaw
          .map((item) => AcceleratorBackend.fromWire(item as String))
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

