import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class DashboardScreen extends StatelessWidget {
  const DashboardScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final safeGiB = (controller.hardware.safeMemoryLimitBytes /
                (1024 * 1024 * 1024))
            .toStringAsFixed(1);

        return Scaffold(
          appBar: AppBar(
            title: const Text('MiniAI — Local-First Multimodal Platform'),
            actions: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: Semantics(
                  label: controller.localOnlyMode
                      ? 'Privacy Status: Local-Only Privacy Mode Active'
                      : 'Privacy Status: Hybrid Consent Mode',
                  child: Chip(
                    avatar: Icon(
                      controller.localOnlyMode
                          ? Icons.shield
                          : Icons.cloud_sync_outlined,
                      size: 18,
                    ),
                    label: Text(
                      controller.localOnlyMode
                          ? 'LOCAL-ONLY PRIVACY'
                          : 'HYBRID CONSENT-GATED',
                    ),
                  ),
                ),
              ),
            ],
          ),
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildPrivacyAndHardwarePanel(context, safeGiB),
                const SizedBox(height: 16),
                _buildAuditBanner(context),
                const SizedBox(height: 16),
                Text(
                  'Independently Pluggable Modality Adapters (${controller.adapterHealthList.length})',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                ...controller.adapterHealthList.map(
                  (health) => _buildModalityCard(context, health),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildPrivacyAndHardwarePanel(BuildContext context, String safeGiB) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Privacy Governance & Hardware Execution Policy',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'Host: ${controller.hardware.osFamily} (${controller.hardware.arch}) • '
              'Logical Cores: ${controller.hardware.logicalCpuCores} • '
              'Active Backend: ${controller.activeExecutionBackend.displayName} • '
              '85% Admission Ceiling: $safeGiB GiB',
            ),
            const Divider(height: 24),
            SwitchListTile(
              key: const Key('local_only_privacy_switch'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Local-Only Privacy Mode (Default)'),
              subtitle: const Text(
                'Blocks all outbound cloud inference and revokes active cloud consent tokens.',
              ),
              value: controller.localOnlyMode,
              onChanged: controller.setLocalOnlyMode,
            ),
            SwitchListTile(
              key: const Key('force_cpu_only_switch'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Force CPU-Only Baseline Execution'),
              subtitle: const Text(
                'Guarantees deterministic CPU execution even if optional GPUs/NPUs are detected.',
              ),
              value: controller.forceCpuOnly,
              onChanged: controller.setForceCpuOnly,
            ),
            if (!controller.localOnlyMode) ...[
              const SizedBox(height: 8),
              Wrap(
                spacing: 12,
                runSpacing: 8,
                children: [
                  ElevatedButton.icon(
                    key: const Key('grant_consent_button'),
                    onPressed: () {
                      controller.grantCloudConsent(
                        CloudConsentToken(
                          tokenId: 'consent-session-001',
                          modality: controller.selectedModality,
                          targetEndpoint: 'https://compute.miniai.local',
                          maxPayloadBytes: 65536,
                          issuedAtEpochSecs: 1700000000,
                          expiresAtEpochSecs: 1700003600,
                          userAcknowledged: true,
                        ),
                      );
                    },
                    icon: const Icon(Icons.verified_user_outlined),
                    label: Text(
                      'Grant Scoped Consent (${controller.selectedModality.displayName})',
                    ),
                  ),
                  if (controller.activeConsent != null)
                    OutlinedButton.icon(
                      key: const Key('revoke_consent_button'),
                      onPressed: controller.revokeCloudConsent,
                      icon: const Icon(Icons.cancel_outlined),
                      label: const Text('Revoke Cloud Consent'),
                    ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildAuditBanner(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.terminal, size: 20),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              controller.lastAuditMessage,
              key: const Key('audit_log_message'),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildModalityCard(BuildContext context, AdapterHealth health) {
    final isSelected = controller.selectedModality == health.modality;
    return Card(
      key: Key('modality_card_${health.modality.wireValue}'),
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: ListTile(
        selected: isSelected,
        leading: Icon(
          health.ready ? Icons.check_circle_outline : Icons.error_outline,
        ),
        title: Text(health.modality.displayName),
        subtitle: Text('${health.adapterId} • ${health.statusMessage}'),
        trailing: Text(health.activeBackend.wireValue.toUpperCase()),
        onTap: () => controller.selectModality(health.modality),
      ),
    );
  }
}

