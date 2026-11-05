import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  late final TextEditingController _workspaceController;
  late final TextEditingController _ipcUrlController;
  late final TextEditingController _nonceController;

  @override
  void initState() {
    super.initState();
    _workspaceController = TextEditingController(
      text: widget.controller.activeWorkspaceId,
    );
    _ipcUrlController = TextEditingController(
      text: widget.controller.ipcServerUrl,
    );
    _nonceController = TextEditingController(
      text: widget.controller.sessionNonce,
    );
  }

  @override
  void didUpdateWidget(SettingsScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.controller.activeWorkspaceId != _workspaceController.text) {
      _workspaceController.text = widget.controller.activeWorkspaceId;
    }
  }

  @override
  void dispose() {
    _workspaceController.dispose();
    _ipcUrlController.dispose();
    _nonceController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final backend = widget.controller.activeExecutionBackend;
        final safeGiB = (widget.controller.hardware.safeMemoryLimitBytes /
                (1024 * 1024 * 1024))
            .toStringAsFixed(1);

        return Scaffold(
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildSectionHeader(
                  context,
                  title: 'Privacy & Cloud Governance',
                  subtitle:
                      'Local-first privacy policy and explicit cloud consent token controls.',
                  icon: Icons.shield_outlined,
                ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      children: [
                        SwitchListTile(
                          key: const Key('settings_local_only_switch'),
                          title: const Text('Local-Only Privacy Mode (Default)'),
                          subtitle: const Text(
                            'Strictly prohibits outbound cloud communication and revokes active consent tokens.',
                          ),
                          value: widget.controller.localOnlyMode,
                          onChanged: widget.controller.setLocalOnlyMode,
                        ),
                        const Divider(),
                        ListTile(
                          leading: Icon(
                            widget.controller.activeConsent != null
                                ? Icons.verified_user
                                : Icons.gpp_bad_outlined,
                            color: widget.controller.activeConsent != null
                                ? Colors.green
                                : Colors.grey,
                          ),
                          title: const Text('Active Cloud Consent Token'),
                          subtitle: Text(
                            widget.controller.activeConsent != null
                                ? 'Token ID: ${widget.controller.activeConsent!.tokenId} '
                                    '(${widget.controller.activeConsent!.modality.displayName})'
                                : 'No active cloud consent token granted.',
                          ),
                          trailing: widget.controller.activeConsent != null
                              ? TextButton(
                                  onPressed:
                                      widget.controller.revokeCloudConsent,
                                  child: const Text('Revoke'),
                                )
                              : null,
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                _buildSectionHeader(
                  context,
                  title: 'Hardware & Compute Backend',
                  subtitle:
                      'Deterministic CPU-first fallback and accelerator detection.',
                  icon: Icons.memory_outlined,
                ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        SwitchListTile(
                          key: const Key('settings_force_cpu_switch'),
                          title: const Text('Force CPU-Only Baseline Execution'),
                          subtitle: const Text(
                            'Guarantees deterministic CPU execution even if GPUs or NPUs are present.',
                          ),
                          value: widget.controller.forceCpuOnly,
                          onChanged: widget.controller.setForceCpuOnly,
                        ),
                        const Divider(),
                        Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 16,
                            vertical: 8,
                          ),
                          child: Text(
                            'Active Backend: ${backend.displayName} • '
                            'Cores: ${widget.controller.hardware.logicalCpuCores} • '
                            '85% Safe RAM Limit: $safeGiB GiB',
                            style: Theme.of(context).textTheme.bodyMedium,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                _buildSectionHeader(
                  context,
                  title: 'Workspace Environment',
                  subtitle:
                      'Active workspace partition persisted in transactional storage.',
                  icon: Icons.folder_open_outlined,
                ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Current Active Workspace: "${widget.controller.activeWorkspaceId}"',
                          style: Theme.of(context).textTheme.titleSmall,
                        ),
                        const SizedBox(height: 12),
                        Row(
                          children: [
                            Expanded(
                              child: TextField(
                                key: const Key('settings_workspace_input'),
                                controller: _workspaceController,
                                decoration: const InputDecoration(
                                  labelText: 'Workspace Identifier',
                                  border: OutlineInputBorder(),
                                  isDense: true,
                                ),
                              ),
                            ),
                            const SizedBox(width: 12),
                            ElevatedButton.icon(
                              key: const Key('settings_save_workspace_button'),
                              onPressed: () {
                                widget.controller.setActiveWorkspace(
                                  _workspaceController.text,
                                );
                              },
                              icon: const Icon(Icons.check),
                              label: const Text('Switch'),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                _buildSectionHeader(
                  context,
                  title: 'Local IPC Server & Authentication',
                  subtitle:
                      'Loopback HTTP/SSE IPC server coordinates storage, events, and adapters.',
                  icon: Icons.lan_outlined,
                ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(
                              widget.controller.ipcConnected
                                  ? Icons.cloud_done
                                  : Icons.cloud_off,
                              color: widget.controller.ipcConnected
                                  ? Colors.green
                                  : Colors.orange,
                              size: 18,
                            ),
                            const SizedBox(width: 8),
                            Text(
                              widget.controller.ipcConnected
                                  ? 'Connected to IPC Server'
                                  : 'IPC Server Standalone / Offline',
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                          ],
                        ),
                        const SizedBox(height: 12),
                        TextField(
                          key: const Key('settings_ipc_url_input'),
                          controller: _ipcUrlController,
                          decoration: const InputDecoration(
                            labelText: 'IPC Base URL (Loopback Only)',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                        const SizedBox(height: 8),
                        TextField(
                          key: const Key('settings_ipc_nonce_input'),
                          controller: _nonceController,
                          obscureText: true,
                          decoration: const InputDecoration(
                            labelText: 'Session Authentication Bearer Nonce',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                        const SizedBox(height: 12),
                        Wrap(
                          spacing: 12,
                          children: [
                            ElevatedButton.icon(
                              key: const Key('settings_save_ipc_button'),
                              onPressed: () {
                                widget.controller.setIpcConfig(
                                  url: _ipcUrlController.text,
                                  nonce: _nonceController.text,
                                );
                              },
                              icon: const Icon(Icons.save),
                              label: const Text('Save IPC Config'),
                            ),
                            OutlinedButton.icon(
                              key: const Key('settings_test_ipc_button'),
                              onPressed: () =>
                                  widget.controller.checkIpcHealth(),
                              icon: const Icon(Icons.wifi_tethering),
                              label: const Text('Test Connection'),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                _buildSectionHeader(
                  context,
                  title: 'Persistence & Diagnostics',
                  subtitle:
                      'Verify offline persistence and emit diagnostic test events.',
                  icon: Icons.bug_report_outlined,
                ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Wrap(
                      spacing: 12,
                      runSpacing: 8,
                      children: [
                        OutlinedButton.icon(
                          key: const Key('settings_reload_button'),
                          onPressed: () => widget.controller.loadSettings(),
                          icon: const Icon(Icons.refresh),
                          label: const Text('Reload Saved Settings'),
                        ),
                        ElevatedButton.icon(
                          key: const Key('settings_diagnostic_ping_button'),
                          onPressed: () => widget.controller.emitDiagnosticPing(
                            'Manual diagnostic ping from Settings panel',
                          ),
                          icon: const Icon(Icons.sensors),
                          label: const Text('Emit Diagnostic Ping'),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildSectionHeader(
    BuildContext context, {
    required String title,
    required String subtitle,
    required IconData icon,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 22, color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                Text(
                  subtitle,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

