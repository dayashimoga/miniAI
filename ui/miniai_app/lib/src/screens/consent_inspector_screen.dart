import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

/// Screen for inspecting, requesting, and managing cryptographic cloud consent tokens,
/// executing hybrid compute dispatches, and monitoring the immutable audit ledger.
class ConsentInspectorScreen extends StatefulWidget {
  const ConsentInspectorScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<ConsentInspectorScreen> createState() => _ConsentInspectorScreenState();
}

class _ConsentInspectorScreenState extends State<ConsentInspectorScreen> {
  // Token creation form controllers
  late final TextEditingController _endpointController;
  late final TextEditingController _payloadBytesController;
  late final TextEditingController _durationSecsController;
  Modality _grantModality = Modality.llm;
  bool _userAcknowledged = false;

  // Hybrid dispatch simulator form controllers
  late final TextEditingController _taskIdController;
  late final TextEditingController _tokenSelectController;
  late final TextEditingController _operationController;
  late final TextEditingController _taskPayloadController;
  Modality _taskModality = Modality.llm;
  bool _simulate500 = false;
  bool _simulateTimeout = false;

  @override
  void initState() {
    super.initState();
    _endpointController = TextEditingController(
      text: 'https://cloud.miniai.internal/v1/compute',
    );
    _payloadBytesController = TextEditingController(text: '1048576'); // 1 MB
    _durationSecsController = TextEditingController(text: '3600'); // 1 hour

    _taskIdController = TextEditingController(
      text: 'task_${DateTime.now().millisecondsSinceEpoch}',
    );
    _tokenSelectController = TextEditingController();
    _operationController = TextEditingController(text: 'inference');
    _taskPayloadController = TextEditingController(text: 'Sample input prompt');
  }

  @override
  void dispose() {
    _endpointController.dispose();
    _payloadBytesController.dispose();
    _durationSecsController.dispose();
    _taskIdController.dispose();
    _tokenSelectController.dispose();
    _operationController.dispose();
    _taskPayloadController.dispose();
    super.dispose();
  }

  void _showGrantConsentDialog() {
    showDialog<void>(
      context: context,
      builder: (dialogCtx) => StatefulBuilder(
        builder: (context, setDialogState) {
          return AlertDialog(
            key: const Key('consent_grant_dialog'),
            title: const Row(
              children: [
                Icon(Icons.vpn_key_outlined, color: Colors.blue),
                SizedBox(width: 8),
                Text('Grant Scoped Cloud Consent'),
              ],
            ),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Explicit user acknowledgement and HTTPS endpoint are mandatory by policy. '
                    'Tokens are cryptographically bound to one modality and time-window.',
                    style: TextStyle(fontSize: 13, color: Colors.grey),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('consent_endpoint_input'),
                    controller: _endpointController,
                    decoration: const InputDecoration(
                      labelText: 'Target HTTPS Endpoint',
                      prefixIcon: Icon(Icons.link),
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<Modality>(
                    key: const Key('consent_modality_dropdown'),
                    isExpanded: true,
                    initialValue: _grantModality,
                    decoration: const InputDecoration(
                      labelText: 'Scoped Modality',
                      border: OutlineInputBorder(),
                    ),
                    items: Modality.values.map((m) {
                      return DropdownMenuItem(
                        value: m,
                        child: Text(
                          m.displayName,
                          overflow: TextOverflow.ellipsis,
                        ),
                      );
                    }).toList(),
                    onChanged: (val) {
                      if (val != null) {
                        setDialogState(() => _grantModality = val);
                      }
                    },
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('consent_max_payload_input'),
                    controller: _payloadBytesController,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(
                      labelText: 'Max Payload Limit (Bytes)',
                      prefixIcon: Icon(Icons.data_usage),
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('consent_duration_input'),
                    controller: _durationSecsController,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(
                      labelText: 'Validity Duration (Seconds)',
                      prefixIcon: Icon(Icons.timer_outlined),
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  CheckboxListTile(
                    key: const Key('consent_ack_checkbox'),
                    title: const Text(
                      'I explicitly authorize outbound transmission to this remote endpoint within stated limits.',
                      style: TextStyle(fontSize: 13),
                    ),
                    value: _userAcknowledged,
                    onChanged: (val) {
                      setDialogState(() => _userAcknowledged = val ?? false);
                    },
                    controlAffinity: ListTileControlAffinity.leading,
                    contentPadding: EdgeInsets.zero,
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(dialogCtx).pop(),
                child: const Text('Cancel'),
              ),
              FilledButton(
                key: const Key('grant_consent_submit_button'),
                onPressed: () {
                  final nowSecs = DateTime.now().millisecondsSinceEpoch ~/ 1000;
                  final duration =
                      int.tryParse(_durationSecsController.text) ?? 3600;
                  final maxBytes =
                      int.tryParse(_payloadBytesController.text) ?? 1048576;
                  final token = CloudConsentToken(
                    tokenId:
                        'tok_${_grantModality.wireValue}_${DateTime.now().millisecondsSinceEpoch}',
                    modality: _grantModality,
                    targetEndpoint: _endpointController.text.trim(),
                    maxPayloadBytes: maxBytes,
                    issuedAtEpochSecs: nowSecs,
                    expiresAtEpochSecs: nowSecs + duration,
                    userAcknowledged: _userAcknowledged,
                  );

                  final success =
                      widget.controller.grantCloudConsent(token);
                  Navigator.of(dialogCtx).pop();

                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text(
                        success
                            ? 'Consent token granted: ${token.tokenId}'
                            : widget.controller.lastAuditMessage,
                      ),
                      backgroundColor: success ? Colors.green : Colors.red,
                    ),
                  );
                },
                child: const Text('Grant Consent'),
              ),
            ],
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final localOnly = widget.controller.localOnlyMode;
        final managedTokens = widget.controller.managedTokens;
        final auditHistory = widget.controller.consentAuditHistory;
        final lastResponse = widget.controller.lastCloudResponse;

        return Scaffold(
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Top Header Card
                _buildHeaderCard(context, localOnly),
                const SizedBox(height: 16),

                // Active Managed Tokens Section
                _buildManagedTokensCard(context, managedTokens, localOnly),
                const SizedBox(height: 16),

                // Hybrid Dispatch Simulator Card
                _buildDispatchSimulatorCard(context, managedTokens, lastResponse),
                const SizedBox(height: 16),

                // Immutable Audit Ledger Table
                _buildAuditLedgerCard(context, auditHistory),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildHeaderCard(BuildContext context, bool localOnly) {
    return Card(
      elevation: 2,
      color: localOnly ? Colors.blueGrey.shade900 : Colors.indigo.shade900,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              spacing: 12,
              runSpacing: 12,
              crossAxisAlignment: WrapCrossAlignment.center,
              alignment: WrapAlignment.spaceBetween,
              children: [
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      localOnly ? Icons.shield : Icons.cloud_done_outlined,
                      color: localOnly
                          ? Colors.lightBlueAccent
                          : Colors.amberAccent,
                      size: 28,
                    ),
                    const SizedBox(width: 12),
                    ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 280),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            localOnly
                                ? 'LOCAL-ONLY PRIVACY (ACTIVE)'
                                : 'HYBRID COMPUTE (CONSENT-GATED)',
                            style: const TextStyle(
                              fontSize: 14,
                              fontWeight: FontWeight.bold,
                              color: Colors.white,
                            ),
                          ),
                          Text(
                            localOnly
                                ? 'Zero outbound egress allowed.'
                                : 'Cloud execution within valid tokens.',
                            style: const TextStyle(
                              fontSize: 11,
                              color: Colors.white70,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
                FilledButton.icon(
                  key: const Key('consent_emergency_kill_switch_button'),
                  style: FilledButton.styleFrom(
                    backgroundColor: Colors.redAccent.shade700,
                    foregroundColor: Colors.white,
                  ),
                  icon: const Icon(Icons.power_settings_new),
                  label: const Text('KILL SWITCH'),
                  onPressed: () {
                    widget.controller.emergencyKillSwitch(
                      reason: 'User activated emergency cloud kill-switch',
                    );
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text(
                          'Emergency Kill-Switch: All tokens revoked, Local-Only Mode enforced.',
                        ),
                        backgroundColor: Colors.red,
                      ),
                    );
                  },
                ),
              ],
            ),
            const Divider(color: Colors.white24, height: 24),
            SwitchListTile(
              key: const Key('consent_inspector_local_only_switch'),
              contentPadding: EdgeInsets.zero,
              title: const Text(
                'Local-Only Privacy Policy Switch:',
                style: TextStyle(color: Colors.white, fontSize: 13),
              ),
              value: localOnly,
              onChanged: (val) {
                widget.controller.setLocalOnlyMode(val);
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildManagedTokensCard(
    BuildContext context,
    List<ManagedConsentTokenContract> tokens,
    bool localOnly,
  ) {
    return Card(
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
                  spacing: 8,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    const Icon(Icons.vpn_key_outlined, color: Colors.indigo),
                    Text(
                      'Scoped Tokens (${tokens.length})',
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                FilledButton.icon(
                  key: const Key('open_consent_dialog_button'),
                  icon: const Icon(Icons.add),
                  label: const Text('Request Consent'),
                  onPressed: _showGrantConsentDialog,
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (tokens.isEmpty)
              Container(
                key: const Key('no_tokens_banner'),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.grey.withAlpha(25),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Center(
                  child: Text(
                    'No active or managed cloud consent tokens found.',
                    style: TextStyle(color: Colors.grey),
                  ),
                ),
              )
            else
              ListView.separated(
                key: const Key('managed_tokens_list'),
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: tokens.length,
                separatorBuilder: (_, __) => const Divider(height: 12),
                itemBuilder: (context, idx) {
                  final item = tokens[idx];
                  final nowSecs = DateTime.now().millisecondsSinceEpoch ~/ 1000;
                  final remainingSecs = item.token.expiresAtEpochSecs - nowSecs;
                  final isExpired = remainingSecs <= 0;

                  return ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: CircleAvatar(
                      backgroundColor: item.isActive && !isExpired
                          ? Colors.green.withAlpha(30)
                          : Colors.grey.withAlpha(30),
                      child: Icon(
                        item.isActive && !isExpired
                            ? Icons.verified_user
                            : Icons.block,
                        color: item.isActive && !isExpired
                            ? Colors.green
                            : Colors.grey,
                      ),
                    ),
                    title: Wrap(
                      spacing: 8,
                      runSpacing: 4,
                      crossAxisAlignment: WrapCrossAlignment.center,
                      children: [
                        Text(
                          item.token.tokenId,
                          style: const TextStyle(fontWeight: FontWeight.bold),
                        ),
                        Chip(
                          visualDensity: VisualDensity.compact,
                          label: Text(item.token.modality.displayName),
                        ),
                        Chip(
                          visualDensity: VisualDensity.compact,
                          backgroundColor: item.isActive && !isExpired
                              ? Colors.green.shade100
                              : Colors.red.shade100,
                          label: Text(
                            item.isActive && !isExpired
                                ? 'ACTIVE (${remainingSecs}s left)'
                                : item.status.toUpperCase(),
                            style: TextStyle(
                              fontSize: 11,
                              color: item.isActive && !isExpired
                                  ? Colors.green.shade900
                                  : Colors.red.shade900,
                            ),
                          ),
                        ),
                      ],
                    ),
                    subtitle: Text(
                      'Endpoint: ${item.token.targetEndpoint}\n'
                      'Quota: ${item.cumulativePayloadBytesUsed} / ${item.token.maxPayloadBytes} Bytes'
                      '${item.revocationReason != null ? " (Reason: ${item.revocationReason})" : ""}',
                      style: const TextStyle(fontSize: 12),
                    ),
                    trailing: item.isActive
                        ? OutlinedButton(
                            key: Key('revoke_token_button_${item.token.tokenId}'),
                            onPressed: () {
                              widget.controller.revokeCloudConsentToken(
                                item.token.tokenId,
                                reason: 'Revoked via Consent Inspector UI',
                              );
                            },
                            child: const Text('Revoke'),
                          )
                        : null,
                  );
                },
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildDispatchSimulatorCard(
    BuildContext context,
    List<ManagedConsentTokenContract> tokens,
    ComputeTaskResponseContract? lastResponse,
  ) {
    return Card(
      key: const Key('hybrid_dispatch_simulator_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Row(
              children: [
                Icon(Icons.bolt, color: Colors.amber),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Hybrid Compute Task Dispatcher',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 6),
            const Text(
              'Simulate remote GPU compute dispatch with automatic CPU fallback when offline, unconsented, or faulted.',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('dispatch_task_id_input'),
                    controller: _taskIdController,
                    decoration: const InputDecoration(
                      labelText: 'Task ID',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    key: const Key('dispatch_token_id_input'),
                    controller: _tokenSelectController,
                    decoration: InputDecoration(
                      labelText: 'Token ID',
                      hintText: tokens.isNotEmpty ? tokens.first.token.tokenId : 'tok_...',
                      border: const OutlineInputBorder(),
                      suffixIcon: tokens.isNotEmpty
                          ? IconButton(
                              icon: const Icon(Icons.arrow_drop_down),
                              onPressed: () {
                                setState(() {
                                  _tokenSelectController.text =
                                      tokens.first.token.tokenId;
                                });
                              },
                            )
                          : null,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: DropdownButtonFormField<Modality>(
                    key: const Key('dispatch_modality_dropdown'),
                    isExpanded: true,
                    initialValue: _taskModality,
                    decoration: const InputDecoration(
                      labelText: 'Task Modality',
                      border: OutlineInputBorder(),
                    ),
                    items: Modality.values.map((m) {
                      return DropdownMenuItem(
                        value: m,
                        child: Text(
                          m.displayName,
                          overflow: TextOverflow.ellipsis,
                        ),
                      );
                    }).toList(),
                    onChanged: (val) {
                      if (val != null) setState(() => _taskModality = val);
                    },
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    key: const Key('dispatch_operation_input'),
                    controller: _operationController,
                    decoration: const InputDecoration(
                      labelText: 'Operation Name',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('dispatch_payload_input'),
              controller: _taskPayloadController,
              decoration: const InputDecoration(
                labelText: 'Payload Data',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            CheckboxListTile(
              key: const Key('simulate_remote_failure_checkbox'),
              title: const Text('Simulate Remote 500 Failure', style: TextStyle(fontSize: 12)),
              value: _simulate500,
              onChanged: (val) => setState(() => _simulate500 = val ?? false),
              controlAffinity: ListTileControlAffinity.leading,
              contentPadding: EdgeInsets.zero,
            ),
            CheckboxListTile(
              key: const Key('simulate_remote_timeout_checkbox'),
              title: const Text('Simulate Remote Timeout', style: TextStyle(fontSize: 12)),
              value: _simulateTimeout,
              onChanged: (val) => setState(() => _simulateTimeout = val ?? false),
              controlAffinity: ListTileControlAffinity.leading,
              contentPadding: EdgeInsets.zero,
            ),
            const SizedBox(height: 12),
            FilledButton.icon(
              key: const Key('dispatch_compute_task_button'),
              icon: const Icon(Icons.send),
              label: const Text('Dispatch Task'),
              onPressed: () async {
                final tokenId = _tokenSelectController.text.trim().isNotEmpty
                    ? _tokenSelectController.text.trim()
                    : (tokens.isNotEmpty ? tokens.first.token.tokenId : 'tok_default');

                final req = ComputeTaskRequestContract(
                  taskId: _taskIdController.text.trim(),
                  tokenId: tokenId,
                  modality: _taskModality,
                  operation: _operationController.text.trim(),
                  payload: _taskPayloadController.text,
                  timeoutMs: 3000,
                );

                await widget.controller.dispatchHybridComputeTask(
                  req,
                  simulateRemoteFailure: _simulate500,
                  simulateRemoteTimeout: _simulateTimeout,
                );
              },
            ),
            if (lastResponse != null) ...[
              const SizedBox(height: 16),
              Container(
                key: const Key('task_result_card'),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: lastResponse.executedRemotely
                      ? Colors.teal.withAlpha(20)
                      : Colors.orange.withAlpha(20),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: lastResponse.executedRemotely
                        ? Colors.teal
                        : Colors.orange,
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Wrap(
                      alignment: WrapAlignment.spaceBetween,
                      crossAxisAlignment: WrapCrossAlignment.center,
                      spacing: 8,
                      runSpacing: 4,
                      children: [
                        Text(
                          'Task Execution Result: ${lastResponse.taskId}',
                          style: const TextStyle(fontWeight: FontWeight.bold),
                        ),
                        Chip(
                          key: const Key('task_execution_badge'),
                          backgroundColor: lastResponse.executedRemotely
                              ? Colors.teal.shade100
                              : Colors.orange.shade100,
                          label: Text(
                            lastResponse.executedRemotely
                                ? 'REMOTE GPU WORKER'
                                : 'LOCAL CPU FALLBACK',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                              color: lastResponse.executedRemotely
                                  ? Colors.teal.shade900
                                  : Colors.orange.shade900,
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 6),
                    Text(
                      'Worker: ${lastResponse.workerId} | Status: ${lastResponse.status} | Latency: ${lastResponse.latencyMs}ms',
                      style: const TextStyle(fontSize: 12),
                    ),
                    Text(
                      'Output: ${lastResponse.output}',
                      style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                    ),
                    if (lastResponse.fallbackReason != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Text(
                          'Fallback Reason: ${lastResponse.fallbackReason}',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.red.shade700,
                            fontWeight: FontWeight.w600,
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
    );
  }

  Widget _buildAuditLedgerCard(
    BuildContext context,
    List<ConsentAuditRecordContract> records,
  ) {
    return Card(
      key: const Key('consent_audit_ledger_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Row(
                    children: [
                      const Icon(Icons.history_edu, color: Colors.brown),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          'Immutable Consent Audit Ledger (${records.length})',
                          style: const TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.bold,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
                if (records.isNotEmpty)
                  TextButton.icon(
                    key: const Key('clear_audit_history_button'),
                    icon: const Icon(Icons.delete_outline, size: 16),
                    label: const Text('Clear View'),
                    onPressed: widget.controller.clearConsentAuditHistory,
                  ),
              ],
            ),
            const SizedBox(height: 8),
            const Text(
              'Append-only record of all consent transitions, pre-flight verifications, and fallback events.',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
            const SizedBox(height: 12),
            if (records.isEmpty)
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.grey.withAlpha(25),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Center(
                  child: Text(
                    'No consent audit events recorded yet.',
                    style: TextStyle(color: Colors.grey),
                  ),
                ),
              )
            else
              ListView.separated(
                key: const Key('consent_audit_ledger_list'),
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: records.length,
                separatorBuilder: (_, __) => const Divider(height: 8),
                itemBuilder: (context, idx) {
                  final record = records[idx];
                  Color actionColor;
                  switch (record.action) {
                    case ConsentActionContract.granted:
                      actionColor = Colors.green;
                      break;
                    case ConsentActionContract.payloadDispatched:
                      actionColor = Colors.blue;
                      break;
                    case ConsentActionContract.rejected:
                      actionColor = Colors.orange;
                      break;
                    case ConsentActionContract.revoked:
                    case ConsentActionContract.expired:
                      actionColor = Colors.red;
                      break;
                    case ConsentActionContract.verified:
                      actionColor = Colors.teal;
                      break;
                  }

                  return ListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    leading: Chip(
                      visualDensity: VisualDensity.compact,
                      backgroundColor: actionColor.withAlpha(30),
                      label: Text(
                        record.action.wireValue.toUpperCase(),
                        style: TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          color: actionColor,
                        ),
                      ),
                    ),
                    title: Text(
                      'Token: ${record.tokenId} (${record.modality.displayName})',
                      style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold),
                    ),
                    subtitle: Text(
                      'Endpoint: ${record.targetEndpoint} | Size: ${record.payloadBytes} B\n'
                      'Details: ${record.detailsJson ?? "N/A"}',
                      style: const TextStyle(fontSize: 11),
                    ),
                    trailing: Text(
                      'T+${record.epochSecs}s',
                      style: const TextStyle(fontSize: 10, color: Colors.grey),
                    ),
                  );
                },
              ),
          ],
        ),
      ),
    );
  }
}
