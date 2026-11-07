import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';

class HardwareInspectorScreen extends StatefulWidget {
  const HardwareInspectorScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<HardwareInspectorScreen> createState() =>
      _HardwareInspectorScreenState();
}

class _HardwareInspectorScreenState extends State<HardwareInspectorScreen> {
  final _testMemoryController = TextEditingController(text: '2048');
  bool? _admissionTestResult;
  int? _lastTestedBytes;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final controller = widget.controller;
        final hw = controller.hardware;

        return Scaffold(
          appBar: AppBar(
            title: const Text('Hardware Capability & Admission Inspector'),
          ),
          body: ListView(
            padding: const EdgeInsets.all(16),
            children: [
          // 1. Host Architecture & Backends
          Card(
            key: const Key('hardware_overview_card'),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.computer, size: 24),
                      const SizedBox(width: 8),
                      Text(
                        'Host System Profile',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      Chip(
                        avatar: const Icon(Icons.terminal, size: 16),
                        label: Text('OS: ${hw.osFamily.toUpperCase()}'),
                      ),
                      Chip(
                        avatar: const Icon(Icons.memory, size: 16),
                        label: Text('Arch: ${hw.arch}'),
                      ),
                      Chip(
                        avatar: const Icon(Icons.speed, size: 16),
                        label: Text('CPU Cores: ${hw.logicalCpuCores}'),
                      ),
                      Chip(
                        avatar: const Icon(Icons.bolt, size: 16),
                        label: Text(
                          'Active Backend: ${controller.activeExecutionBackend.displayName}',
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Text(
                    'Detected Hardware Accelerator Backends:',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 8,
                    children: [
                      for (final backend in hw.detectedBackends)
                        Chip(
                          key: Key('detected_backend_${backend.wireValue}'),
                          avatar: Icon(
                            backend.isCpu ? Icons.laptop : Icons.rocket_launch,
                            size: 16,
                            color: backend.isCpu ? Colors.blue : Colors.green,
                          ),
                          label: Text(backend.displayName),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Live Resource Telemetry & Active Model in RAM
          Card(
            key: const Key('live_telemetry_card'),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.show_chart, size: 24),
                      const SizedBox(width: 8),
                      Text(
                        'Live Resource Telemetry & Active Model',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Loaded Model in RAM:',
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                            const SizedBox(height: 4),
                            Text(
                              controller.activeLoadedModel ?? 'None (Memory Unloaded)',
                              style: Theme.of(context).textTheme.titleSmall?.copyWith(
                                    fontWeight: FontWeight.bold,
                                    color: controller.activeLoadedModel != null
                                        ? Colors.tealAccent
                                        : Colors.grey,
                                  ),
                            ),
                          ],
                        ),
                      ),
                      if (controller.activeLoadedModel != null)
                        FilledButton.tonal(
                          key: const Key('unload_model_button'),
                          onPressed: () => controller.unloadActiveModel(),
                          child: const Text('Unload Model from RAM'),
                        ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      Chip(
                        avatar: const Icon(Icons.storage, size: 16),
                        label: Text('Peak Memory: ${controller.peakMemoryRssMb.toStringAsFixed(1)} MB'),
                      ),
                      Chip(
                        avatar: const Icon(Icons.timer, size: 16),
                        label: Text('Avg TTFT: ${controller.avgTtftMs.toStringAsFixed(1)} ms'),
                      ),
                      Chip(
                        avatar: const Icon(Icons.speed, size: 16),
                        label: Text('Throughput: ${controller.avgTokPerSec.toStringAsFixed(1)} tok/s'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          Card(
            key: const Key('memory_admission_card'),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.security, size: 24),
                      const SizedBox(width: 8),
                      Text(
                        '85% Memory Ceiling Admission Guard',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Total RAM: ${(hw.totalMemoryBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB',
                      ),
                      Text(
                        'Available: ${(hw.availableMemoryBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB',
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  LinearProgressIndicator(
                    key: const Key('ram_ceiling_indicator'),
                    value: 0.85,
                    color: Colors.amber,
                    backgroundColor: Colors.grey.shade300,
                  ),
                  const SizedBox(height: 4),
                  Align(
                    alignment: Alignment.centerRight,
                    child: Text(
                      '85% Safe Admission Limit: ${(hw.safeMemoryLimitBytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.amber.shade900,
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  const Divider(),
                  const SizedBox(height: 8),
                  Text(
                    'Test Model Memory Admission:',
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      Expanded(
                        child: TextField(
                          key: const Key('test_admission_input'),
                          controller: _testMemoryController,
                          keyboardType: TextInputType.number,
                          decoration: const InputDecoration(
                            labelText: 'Required Model Footprint (MB)',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),
                      ElevatedButton(
                        key: const Key('test_admission_button'),
                        onPressed: () {
                          final mb = int.tryParse(_testMemoryController.text.trim()) ?? 0;
                          final bytes = mb * 1024 * 1024;
                          setState(() {
                            _lastTestedBytes = bytes;
                            _admissionTestResult = controller.testMemoryAdmission(bytes);
                          });
                        },
                        child: const Text('Evaluate'),
                      ),
                    ],
                  ),
                  if (_admissionTestResult != null) ...[
                    const SizedBox(height: 12),
                    Container(
                      key: const Key('admission_result_banner'),
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: _admissionTestResult!
                            ? Colors.green.shade50
                            : Colors.red.shade50,
                        border: Border.all(
                          color: _admissionTestResult! ? Colors.green : Colors.red,
                        ),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            _admissionTestResult!
                                ? Icons.check_circle
                                : Icons.error_outline,
                            color: _admissionTestResult! ? Colors.green : Colors.red,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              _admissionTestResult!
                                  ? 'ADMITTED: ${(_lastTestedBytes! / (1024 * 1024)).toStringAsFixed(0)} MB is safely within the 85% memory ceiling.'
                                  : 'REJECTED: ${(_lastTestedBytes! / (1024 * 1024)).toStringAsFixed(0)} MB exceeds the 85% safe memory limit (${(hw.safeMemoryLimitBytes / (1024 * 1024)).toStringAsFixed(0)} MB).',
                              style: TextStyle(
                                color: _admissionTestResult!
                                    ? Colors.green.shade900
                                    : Colors.red.shade900,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          // 3. CPU SIMD Capabilities
          Card(
            key: const Key('cpu_simd_card'),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.extension, size: 24),
                      const SizedBox(width: 8),
                      Text(
                        'CPU SIMD Instruction Sets',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  if (hw.cpuFeatures.enabledFeatures.isEmpty)
                    const Text('Universal Standard CPU Instruction Baseline Active')
                  else
                    Wrap(
                      spacing: 8,
                      runSpacing: 8,
                      children: [
                        for (final feat in hw.cpuFeatures.enabledFeatures)
                          Chip(
                            key: Key('simd_feature_$feat'),
                            avatar: const Icon(Icons.flash_on, size: 16, color: Colors.amber),
                            label: Text(feat),
                          ),
                      ],
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          // 4. GPU / Accelerator Devices
          Card(
            key: const Key('gpu_devices_card'),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.videogame_asset, size: 24),
                      const SizedBox(width: 8),
                      Text(
                        'GPU & Neural Accelerator Devices',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  if (hw.gpus.isEmpty)
                    const Text(
                      'No discrete accelerator GPU detected. Execution operates via universal CPU-first baseline.',
                    )
                  else
                    for (final gpu in hw.gpus) ...[
                      ListTile(
                        leading: const Icon(Icons.memory),
                        title: Text(gpu.name),
                        subtitle: Text(
                          'Backend: ${gpu.backend.displayName} • VRAM: ${(gpu.availableVramBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB avail / ${(gpu.totalVramBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB total (85% safe: ${(gpu.safeVramLimitBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB)',
                        ),
                      ),
                    ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
      },
    );
  }
}
