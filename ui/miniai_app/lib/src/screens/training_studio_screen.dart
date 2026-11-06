import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class TrainingStudioScreen extends StatefulWidget {
  const TrainingStudioScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<TrainingStudioScreen> createState() => _TrainingStudioScreenState();
}

class _TrainingStudioScreenState extends State<TrainingStudioScreen> {
  late int _rank;
  late double _loraAlpha;
  late double _learningRate;
  late int _epochs;
  late int _batchSize;
  late int _seed;

  @override
  void initState() {
    super.initState();
    final hp = widget.controller.trainingHyperparameters;
    _rank = hp.rank;
    _loraAlpha = hp.loraAlpha;
    _learningRate = hp.learningRate;
    _epochs = hp.epochs;
    _batchSize = hp.batchSize;
    _seed = hp.seed;
  }

  void _syncHyperparameters() {
    widget.controller.updateTrainingHyperparameters(
      widget.controller.trainingHyperparameters.copyWith(
        rank: _rank,
        loraAlpha: _loraAlpha,
        learningRate: _learningRate,
        epochs: _epochs,
        batchSize: _batchSize,
        seed: _seed,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final ctrl = widget.controller;
        final activeJob = ctrl.activeTrainingJob;
        final isBusy = ctrl.isTrainingActive;

        return Scaffold(
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildHeaderBar(context, ctrl, isBusy),
                const SizedBox(height: 16),
                _buildDatasetSplitInspector(context, ctrl),
                const SizedBox(height: 16),
                _buildHyperparametersCard(context, ctrl, isBusy),
                const SizedBox(height: 16),
                if (activeJob != null) ...[
                  _buildLiveProgressCard(context, ctrl, activeJob, isBusy),
                  const SizedBox(height: 16),
                  if (activeJob.status == TrainingJobStatus.completed)
                    _buildEvaluationComparisonCard(context, ctrl, activeJob),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildHeaderBar(
    BuildContext context,
    PlatformController ctrl,
    bool isBusy,
  ) {
    final theme = Theme.of(context);
    return Card(
      key: const Key('training_studio_header_card'),
      elevation: 0,
      color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Wrap(
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          spacing: 12,
          runSpacing: 12,
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'MiniAI Training Studio',
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  'LoRA / QLoRA Fine-Tuning & Distillation Workbench',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.outline,
                  ),
                ),
              ],
            ),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Chip(
                  key: const Key('training_device_badge'),
                  avatar: const Icon(Icons.memory, size: 16),
                  label: Text('${ctrl.activeExecutionBackend.displayName} Ready'),
                ),
                if (isBusy)
                  FilledButton.tonalIcon(
                    key: const Key('cancel_training_button'),
                    icon: const Icon(Icons.stop_circle_outlined),
                    label: const Text('Cancel Job'),
                    style: FilledButton.styleFrom(
                      foregroundColor: theme.colorScheme.error,
                    ),
                    onPressed: ctrl.cancelTrainingJob,
                  )
                else
                  FilledButton.icon(
                    key: const Key('start_training_button'),
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('Start Fine-Tuning'),
                    onPressed: () => ctrl.startTrainingJob(),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDatasetSplitInspector(
    BuildContext context,
    PlatformController ctrl,
  ) {
    final theme = Theme.of(context);
    final ds = ctrl.selectedDataset;

    return Card(
      key: const Key('dataset_split_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              alignment: WrapAlignment.spaceBetween,
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: 8,
              runSpacing: 8,
              children: [
                Wrap(
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: 8,
                  children: [
                    Icon(Icons.pie_chart_outline, color: theme.colorScheme.primary),
                    Text(
                      'Dataset & Split Inspector',
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                SizedBox(
                  width: 220,
                  child: DropdownButtonHideUnderline(
                    child: DropdownButton<String>(
                      key: const Key('dataset_selector_dropdown'),
                      isDense: true,
                      isExpanded: true,
                      value: ds.name,
                      items: ctrl.availableDatasets
                          .map(
                            (d) => DropdownMenuItem(
                              value: d.name,
                              child: Text(d.name),
                            ),
                          )
                          .toList(),
                      onChanged: (val) {
                        if (val != null) {
                          final item = ctrl.availableDatasets.firstWhere(
                            (d) => d.name == val,
                          );
                          ctrl.setSelectedDataset(item);
                        }
                      },
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _buildSplitChip(
                  theme,
                  label: 'Train: ${ds.trainRecords}',
                  color: Colors.blue.shade100,
                  textColor: Colors.blue.shade900,
                ),
                _buildSplitChip(
                  theme,
                  label: 'Val: ${ds.valRecords}',
                  color: Colors.amber.shade100,
                  textColor: Colors.amber.shade900,
                ),
                _buildSplitChip(
                  theme,
                  label: 'Test: ${ds.testRecords}',
                  color: Colors.green.shade100,
                  textColor: Colors.green.shade900,
                ),
                _buildSplitChip(
                  theme,
                  label: 'Total: ${ds.totalRecords}',
                  color: theme.colorScheme.surfaceContainerHighest,
                  textColor: theme.colorScheme.onSurface,
                ),
              ],
            ),
            const SizedBox(height: 8),
            SelectableText(
              key: const Key('dataset_sha_badge'),
              'SHA-256: ${ds.sha256Digest}',
              style: theme.textTheme.bodySmall?.copyWith(
                fontFamily: 'monospace',
                color: theme.colorScheme.outline,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSplitChip(
    ThemeData theme, {
    required String label,
    required Color color,
    required Color textColor,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.bold,
          color: textColor,
        ),
      ),
    );
  }

  Widget _buildHyperparametersCard(
    BuildContext context,
    PlatformController ctrl,
    bool isBusy,
  ) {
    final theme = Theme.of(context);

    return Card(
      key: const Key('hyperparameters_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: 8,
              children: [
                Icon(Icons.tune, color: theme.colorScheme.primary),
                Text(
                  'Hyperparameter Tuning Form',
                  style: theme.textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Method SegmentedButton wrapped in horizontal scroll
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: SegmentedButton<TrainingMethod>(
                key: const Key('method_segmented_button'),
                segments: TrainingMethod.values
                    .map(
                      (m) => ButtonSegment(
                        value: m,
                        label: Text(m.wireValue.toUpperCase()),
                      ),
                    )
                    .toList(),
                selected: {ctrl.selectedTrainingMethod},
                onSelectionChanged: isBusy
                    ? null
                    : (selection) {
                        ctrl.setTrainingMethod(selection.first);
                      },
              ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 16,
              runSpacing: 16,
              children: [
                SizedBox(
                  width: 160,
                  child: DropdownButtonFormField<int>(
                    key: const Key('lora_rank_dropdown'),
                    isDense: true,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Rank (r)',
                      border: OutlineInputBorder(),
                      contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                    initialValue: _rank,
                    items: const [2, 4, 8, 16, 32]
                        .map(
                          (v) => DropdownMenuItem(
                            value: v,
                            child: Text('r = $v'),
                          ),
                        )
                        .toList(),
                    onChanged: isBusy
                        ? null
                        : (val) {
                            if (val != null) {
                              setState(() => _rank = val);
                              _syncHyperparameters();
                            }
                          },
                  ),
                ),
                SizedBox(
                  width: 160,
                  child: DropdownButtonFormField<double>(
                    key: const Key('lora_alpha_dropdown'),
                    isDense: true,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Alpha (α)',
                      border: OutlineInputBorder(),
                      contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                    initialValue: _loraAlpha,
                    items: const [4.0, 8.0, 16.0, 32.0]
                        .map(
                          (v) => DropdownMenuItem(
                            value: v,
                            child: Text('α = ${v.toInt()}'),
                          ),
                        )
                        .toList(),
                    onChanged: isBusy
                        ? null
                        : (val) {
                            if (val != null) {
                              setState(() => _loraAlpha = val);
                              _syncHyperparameters();
                            }
                          },
                  ),
                ),
                SizedBox(
                  width: 160,
                  child: DropdownButtonFormField<int>(
                    key: const Key('epochs_dropdown'),
                    isDense: true,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Epochs',
                      border: OutlineInputBorder(),
                      contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                    initialValue: _epochs,
                    items: const [1, 2, 3, 5, 10]
                        .map(
                          (v) => DropdownMenuItem(
                            value: v,
                            child: Text('$v epochs'),
                          ),
                        )
                        .toList(),
                    onChanged: isBusy
                        ? null
                        : (val) {
                            if (val != null) {
                              setState(() => _epochs = val);
                              _syncHyperparameters();
                            }
                          },
                  ),
                ),
                SizedBox(
                  width: 160,
                  child: DropdownButtonFormField<int>(
                    key: const Key('batch_size_dropdown'),
                    isDense: true,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Batch Size',
                      border: OutlineInputBorder(),
                      contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                    initialValue: _batchSize,
                    items: const [2, 4, 8, 16]
                        .map(
                          (v) => DropdownMenuItem(
                            value: v,
                            child: Text('batch $v'),
                          ),
                        )
                        .toList(),
                    onChanged: isBusy
                        ? null
                        : (val) {
                            if (val != null) {
                              setState(() => _batchSize = val);
                              _syncHyperparameters();
                            }
                          },
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLiveProgressCard(
    BuildContext context,
    PlatformController ctrl,
    TrainingJob job,
    bool isBusy,
  ) {
    final theme = Theme.of(context);

    Color statusColor;
    switch (job.status) {
      case TrainingJobStatus.completed:
        statusColor = Colors.green;
      case TrainingJobStatus.failed:
      case TrainingJobStatus.cancelled:
        statusColor = Colors.red;
      case TrainingJobStatus.training:
      case TrainingJobStatus.evaluating:
      case TrainingJobStatus.preparing:
        statusColor = Colors.blue;
      case TrainingJobStatus.idle:
        statusColor = Colors.grey;
    }

    return Card(
      key: const Key('training_progress_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              alignment: WrapAlignment.spaceBetween,
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: 8,
              runSpacing: 8,
              children: [
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.analytics_outlined, color: theme.colorScheme.primary),
                    const SizedBox(width: 8),
                    Text(
                      'Training Telemetry & Convergence',
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                Chip(
                  key: const Key('job_status_badge'),
                  backgroundColor: statusColor.withValues(alpha: 0.15),
                  side: BorderSide(color: statusColor),
                  label: Text(
                    job.status.displayName,
                    style: TextStyle(color: statusColor, fontWeight: FontWeight.bold),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            LinearProgressIndicator(
              key: const Key('training_progress_bar'),
              value: job.status == TrainingJobStatus.completed ? 1.0 : job.progress,
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 16,
              runSpacing: 8,
              children: [
                _buildMetricItem(
                  'Step',
                  '${job.currentStep} / ${job.totalSteps}',
                  key: const Key('current_step_badge'),
                ),
                _buildMetricItem(
                  'Epoch',
                  '${job.currentEpoch} / ${job.hyperparameters.epochs}',
                  key: const Key('current_epoch_badge'),
                ),
                _buildMetricItem(
                  'Loss',
                  job.currentLoss.toStringAsFixed(4),
                  key: const Key('current_loss_badge'),
                ),
                if (job.status == TrainingJobStatus.completed) ...[
                  _buildMetricItem(
                    'Val Loss',
                    job.valLoss.toStringAsFixed(4),
                    key: const Key('val_loss_badge'),
                  ),
                  _buildMetricItem(
                    'Perplexity',
                    job.perplexity.toStringAsFixed(2),
                    key: const Key('perplexity_badge'),
                  ),
                ],
              ],
            ),
            if (job.lossHistory.isNotEmpty) ...[
              const SizedBox(height: 12),
              Text(
                'Loss History (${job.lossHistory.length} checkpoints):',
                style: theme.textTheme.bodySmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 4),
              SingleChildScrollView(
                key: const Key('loss_history_view'),
                scrollDirection: Axis.horizontal,
                child: Row(
                  children: job.lossHistory
                      .map(
                        (l) => Container(
                          margin: const EdgeInsets.only(right: 6),
                          padding: const EdgeInsets.symmetric(
                            horizontal: 6,
                            vertical: 2,
                          ),
                          decoration: BoxDecoration(
                            color: theme.colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            l.toStringAsFixed(3),
                            style: const TextStyle(fontSize: 11),
                          ),
                        ),
                      )
                      .toList(),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildEvaluationComparisonCard(
    BuildContext context,
    PlatformController ctrl,
    TrainingJob job,
  ) {
    final theme = Theme.of(context);
    final isAlreadyRegistered =
        ctrl.models.any((m) => m.modelId == 'adapter-${job.id}');

    return Card(
      key: const Key('evaluation_comparison_card'),
      color: theme.colorScheme.primaryContainer.withValues(alpha: 0.25),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: 8,
              children: [
                Icon(Icons.compare_arrows, color: theme.colorScheme.primary),
                Text(
                  'Base Model vs Fine-Tuned Adapter Comparison',
                  style: theme.textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: ConstrainedBox(
                constraints: const BoxConstraints(minWidth: 460),
                child: Table(
                  border: TableBorder.all(
                    color: theme.colorScheme.outlineVariant,
                    width: 1,
                  ),
                  children: [
                    const TableRow(
                      children: [
                        Padding(
                          padding: EdgeInsets.all(8),
                          child: Text(
                            'Metric',
                            style: TextStyle(fontWeight: FontWeight.bold),
                          ),
                        ),
                        Padding(
                          padding: EdgeInsets.all(8),
                          child: Text(
                            'Base Model',
                            style: TextStyle(fontWeight: FontWeight.bold),
                          ),
                        ),
                        Padding(
                          padding: EdgeInsets.all(8),
                          child: Text(
                            'Fine-Tuned Adapter',
                            style: TextStyle(fontWeight: FontWeight.bold),
                          ),
                        ),
                        Padding(
                          padding: EdgeInsets.all(8),
                          child: Text(
                            'Delta',
                            style: TextStyle(fontWeight: FontWeight.bold),
                          ),
                        ),
                      ],
                    ),
                    TableRow(
                      children: [
                        const Padding(
                          padding: EdgeInsets.all(8),
                          child: Text('Cross-Entropy Loss'),
                        ),
                        const Padding(
                          padding: EdgeInsets.all(8),
                          child: Text('2.450'),
                        ),
                        Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(job.currentLoss.toStringAsFixed(3)),
                        ),
                        Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(
                            '-${((2.45 - job.currentLoss) / 2.45 * 100).toStringAsFixed(1)}%',
                            style: const TextStyle(
                              color: Colors.green,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ],
                    ),
                    TableRow(
                      children: [
                        const Padding(
                          padding: EdgeInsets.all(8),
                          child: Text('Validation Perplexity'),
                        ),
                        const Padding(
                          padding: EdgeInsets.all(8),
                          child: Text('11.58'),
                        ),
                        Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(job.perplexity.toStringAsFixed(2)),
                        ),
                        Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(
                            '-${((11.58 - job.perplexity) / 11.58 * 100).toStringAsFixed(1)}%',
                            style: const TextStyle(
                              color: Colors.green,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 12,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                FilledButton.icon(
                  key: const Key('register_adapter_button'),
                  icon: Icon(
                    isAlreadyRegistered ? Icons.check : Icons.app_registration,
                  ),
                  label: Text(
                    isAlreadyRegistered
                        ? 'Adapter Registered in Catalog'
                        : 'Register Adapter into Model Catalog',
                  ),
                  onPressed: isAlreadyRegistered
                      ? null
                      : () => ctrl.registerTrainedAdapterAsModel(job),
                ),
                if (job.adapterSha256 != null)
                  Text(
                    'Digest: ${job.adapterSha256!.substring(0, 16)}...',
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetricItem(String label, String value, {Key? key}) {
    return Column(
      key: key,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(fontSize: 11, color: Colors.grey),
        ),
        Text(
          value,
          style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
        ),
      ],
    );
  }
}
