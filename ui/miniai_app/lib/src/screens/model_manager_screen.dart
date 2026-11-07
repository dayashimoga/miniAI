import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class ModelManagerScreen extends StatefulWidget {
  const ModelManagerScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<ModelManagerScreen> createState() => _ModelManagerScreenState();
}

class _ModelManagerScreenState extends State<ModelManagerScreen> {
  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final controller = widget.controller;

        return Scaffold(
          appBar: AppBar(
            title: const Text('Model Registry & Verified Cache'),
            actions: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                child: ElevatedButton.icon(
                  key: const Key('open_import_dialog_button'),
                  icon: const Icon(Icons.file_upload, size: 18),
                  label: const Text('Import Model'),
                  onPressed: () => _showImportDialog(context),
                ),
              ),
            ],
          ),
          body: Column(
            children: [
              _buildModalityFilters(controller),
              const Divider(height: 1),
              Expanded(
                child: controller.filteredModels.isEmpty
                    ? const Center(
                        key: Key('no_models_message'),
                        child: Text('No models match the active filter.'),
                      )
                    : ListView.builder(
                        key: const Key('models_list_view'),
                        padding: const EdgeInsets.all(16),
                        itemCount: controller.filteredModels.length,
                        itemBuilder: (context, index) {
                          final model = controller.filteredModels[index];
                          return _buildModelCard(context, model);
                        },
                      ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildModalityFilters(PlatformController controller) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          FilterChip(
            key: const Key('filter_model_all'),
            label: const Text('All Modalities'),
            selected: controller.modelModalityFilter == null,
            onSelected: (selected) {
              if (selected) controller.setModelModalityFilter(null);
            },
          ),
          const SizedBox(width: 8),
          for (final modality in Modality.values) ...[
            FilterChip(
              key: Key('filter_model_${modality.wireValue}'),
              label: Text(modality.displayName),
              selected: controller.modelModalityFilter == modality,
              onSelected: (selected) {
                controller.setModelModalityFilter(selected ? modality : null);
              },
            ),
            const SizedBox(width: 8),
          ],
        ],
      ),
    );
  }

  Widget _buildModelCard(BuildContext context, ModelManifest model) {
    final controller = widget.controller;
    final progress = controller.getModelStatus(model.modelId);
    final isVerified = progress.status == DownloadStatus.verified;
    final isDownloading = progress.status == DownloadStatus.downloading;
    final isPaused = progress.status == DownloadStatus.paused;
    final isFailed = progress.status == DownloadStatus.failed;

    return Card(
      key: Key('model_card_${model.modelId}'),
      margin: const EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        model.name,
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        model.description,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
                Chip(
                  label: Text(model.modality.displayName),
                  backgroundColor: Theme.of(context).colorScheme.surfaceContainerHighest,
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                Chip(
                  avatar: const Icon(Icons.tag, size: 14),
                  label: Text(model.format.toUpperCase()),
                ),
                if (model.quantization != null)
                  Chip(
                    avatar: const Icon(Icons.compress, size: 14),
                    label: Text(model.quantization!),
                  ),
                if (model.parameterCount != null)
                  Chip(
                    avatar: const Icon(Icons.memory, size: 14),
                    label: Text(model.parameterCount!),
                  ),
                Chip(
                  avatar: const Icon(Icons.storage, size: 14),
                  label: Text(model.formattedSize),
                ),
                Chip(
                  key: Key('sha256_badge_${model.modelId}'),
                  avatar: Icon(
                    isVerified ? Icons.verified : Icons.lock_outline,
                    size: 14,
                    color: isVerified ? Colors.green : null,
                  ),
                  label: Text('SHA-256: ${model.shortSha256}'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (isDownloading || isPaused) ...[
              LinearProgressIndicator(
                key: Key('download_progress_bar_${model.modelId}'),
                value: progress.percentage / 100.0,
              ),
              const SizedBox(height: 6),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    '${progress.percentage.toStringAsFixed(1)}% (${(progress.bytesDownloaded / (1024 * 1024)).toStringAsFixed(1)} MB / ${(progress.totalBytes / (1024 * 1024)).toStringAsFixed(1)} MB)',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  Row(
                    children: [
                      if (isDownloading) ...[
                        TextButton.icon(
                          key: Key('verify_download_${model.modelId}'),
                          icon: const Icon(Icons.check_circle_outline, size: 16),
                          label: const Text('Verify & Finish'),
                          onPressed: () => controller.completeDownload(model.modelId),
                        ),
                        IconButton(
                          key: Key('pause_download_${model.modelId}'),
                          tooltip: 'Pause Download',
                          icon: const Icon(Icons.pause, size: 20),
                          onPressed: () => controller.pauseDownload(model.modelId),
                        ),
                      ],
                      if (isPaused)
                        IconButton(
                          key: Key('resume_download_${model.modelId}'),
                          tooltip: 'Resume Download',
                          icon: const Icon(Icons.play_arrow, size: 20),
                          onPressed: () => controller.resumeDownload(model.modelId),
                        ),
                      IconButton(
                        key: Key('cancel_download_${model.modelId}'),
                        tooltip: 'Cancel Download',
                        icon: const Icon(Icons.close, size: 20),
                        onPressed: () => controller.cancelDownload(model.modelId),
                      ),
                    ],
                  ),
                ],
              ),
            ] else if (isVerified) ...[
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Chip(
                    key: Key('verified_badge'),
                    avatar: Icon(Icons.check_circle, color: Colors.green, size: 16),
                    label: Text('Verified in Cache'),
                  ),
                  IconButton(
                    key: Key('delete_model_${model.modelId}'),
                    tooltip: 'Delete Model',
                    icon: const Icon(Icons.delete_outline, size: 20),
                    onPressed: () => controller.deleteModel(model.modelId),
                  ),
                ],
              ),
            ] else if (isFailed) ...[
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Expanded(
                        child: Text(
                          progress.errorMessage ?? 'Download failed',
                          style: const TextStyle(color: Colors.red),
                        ),
                      ),
                      ElevatedButton.icon(
                        key: Key('retry_download_${model.modelId}'),
                        icon: const Icon(Icons.refresh, size: 16),
                        label: const Text('Retry'),
                        onPressed: () => controller.startDownload(model.modelId),
                      ),
                    ],
                  ),
                  if (controller.localOnlyMode) ...[
                    const SizedBox(height: 6),
                    Row(
                      children: [
                        const Icon(Icons.info_outline, size: 14, color: Colors.orange),
                        const SizedBox(width: 6),
                        Expanded(
                          child: Text(
                            'To download verified open weights from HuggingFace, disable Local-Only Privacy Mode in the top header or allow download below.',
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.orange),
                          ),
                        ),
                        TextButton(
                          key: Key('allow_download_${model.modelId}'),
                          onPressed: () {
                            controller.setLocalOnlyMode(false);
                            controller.startDownload(model.modelId);
                          },
                          child: const Text('Allow & Download'),
                        ),
                      ],
                    ),
                  ],
                ],
              ),
            ] else ...[
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  ElevatedButton.icon(
                    key: Key('start_download_${model.modelId}'),
                    icon: const Icon(Icons.download, size: 16),
                    label: const Text('Download & Verify'),
                    onPressed: () => controller.startDownload(model.modelId),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  void _showImportDialog(BuildContext context) {
    final idController = TextEditingController();
    final nameController = TextEditingController();
    final pathController = TextEditingController();
    final formatController = TextEditingController(text: 'gguf');
    final shaController = TextEditingController();
    var selectedModality = Modality.llm;

    showDialog(
      context: context,
      builder: (ctx) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            return AlertDialog(
              title: const Text('Import Local Model Artifact'),
              content: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      key: const Key('import_model_id_field'),
                      controller: idController,
                      decoration: const InputDecoration(
                        labelText: 'Model ID (e.g. custom-llama-q4)',
                      ),
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      key: const Key('import_model_name_field'),
                      controller: nameController,
                      decoration: const InputDecoration(
                        labelText: 'Display Name',
                      ),
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      key: const Key('import_model_path_field'),
                      controller: pathController,
                      decoration: const InputDecoration(
                        labelText: 'File Path on Host',
                      ),
                    ),
                    const SizedBox(height: 8),
                    DropdownButtonFormField<Modality>(
                      key: const Key('import_modality_dropdown'),
                      initialValue: selectedModality,
                      decoration: const InputDecoration(labelText: 'Modality'),
                      items: Modality.values.map((m) {
                        return DropdownMenuItem(
                          value: m,
                          child: Text(m.displayName),
                        );
                      }).toList(),
                      onChanged: (val) {
                        if (val != null) {
                          setDialogState(() => selectedModality = val);
                        }
                      },
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      key: const Key('import_model_format_field'),
                      controller: formatController,
                      decoration: const InputDecoration(
                        labelText: 'Format (gguf, onnx, safetensors)',
                      ),
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      key: const Key('import_model_sha256_field'),
                      controller: shaController,
                      decoration: const InputDecoration(
                        labelText: 'Expected SHA-256 (Optional)',
                      ),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  key: const Key('import_cancel_button'),
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: const Text('Cancel'),
                ),
                ElevatedButton(
                  key: const Key('import_confirm_button'),
                  onPressed: () {
                    final id = idController.text.trim();
                    final name = nameController.text.trim();
                    final path = pathController.text.trim();
                    final format = formatController.text.trim();
                    final sha = shaController.text.trim();

                    if (id.isEmpty || name.isEmpty || path.isEmpty) {
                      return;
                    }

                    widget.controller.importLocalModel(
                      LocalModelImportRequest(
                        sourcePath: path,
                        modelId: id,
                        name: name,
                        modality: selectedModality,
                        version: '1.0.0',
                        description: 'Locally imported weights artifact',
                        format: format.isEmpty ? 'gguf' : format,
                        minRamBytes: 1024 * 1024 * 1024,
                        recommendedBackend: AcceleratorBackend.cpu,
                        expectedSha256: sha.isEmpty ? null : sha,
                        overwrite: true,
                      ),
                    );
                    Navigator.of(ctx).pop();
                  },
                  child: const Text('Import'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}
