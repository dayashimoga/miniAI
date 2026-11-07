import 'dart:convert';
import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class EventInspectorView extends StatelessWidget {
  const EventInspectorView({
    super.key,
    required this.controller,
    this.isDrawer = false,
  });

  final PlatformController controller;
  final bool isDrawer;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final events = controller.filteredEvents;
        final totalCount = controller.events.length;
        final filter = controller.eventFilter ?? 'all';

        final content = Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildToolbar(context, totalCount, filter),
            const Divider(height: 1),
            Expanded(
              child: events.isEmpty
                  ? _buildEmptyState(context)
                  : ListView.separated(
                      key: const Key('event_list_view'),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 8,
                      ),
                      itemCount: events.length,
                      separatorBuilder: (_, __) => const SizedBox(height: 6),
                      itemBuilder: (context, index) {
                        final event = events[index];
                        return _buildEventCard(context, event);
                      },
                    ),
            ),
          ],
        );

        if (isDrawer) {
          return Drawer(
            child: SafeArea(
              child: content,
            ),
          );
        }

        return Scaffold(
          body: content,
        );
      },
    );
  }

  Widget _buildToolbar(BuildContext context, int totalCount, String currentFilter) {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.stream,
                size: 20,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  isDrawer ? 'Events ($totalCount)' : 'Live System Event Stream ($totalCount)',
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ),
              IconButton(
                key: const Key('emit_ping_icon_button'),
                icon: const Icon(Icons.add_alert_outlined),
                tooltip: 'Emit Diagnostic Event',
                onPressed: () => controller.emitDiagnosticPing(),
              ),
              IconButton(
                key: const Key('export_diagnostics_button'),
                icon: const Icon(Icons.file_download_outlined),
                tooltip: 'Export Redacted Diagnostics',
                onPressed: () {
                  controller.exportRedactedDiagnostics();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Redacted diagnostics bundle exported'),
                      duration: Duration(seconds: 2),
                    ),
                  );
                },
              ),
              IconButton(
                key: const Key('clear_events_button'),
                icon: const Icon(Icons.delete_sweep_outlined),
                tooltip: 'Clear Event History',
                onPressed: () => controller.clearEvents(),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                _buildFilterChip(context, label: 'All', value: 'all', active: currentFilter == 'all'),
                const SizedBox(width: 6),
                _buildFilterChip(
                  context,
                  label: 'Privacy',
                  value: 'privacy_policy_updated',
                  active: currentFilter == 'privacy_policy_updated',
                ),
                const SizedBox(width: 6),
                _buildFilterChip(
                  context,
                  label: 'Hardware',
                  value: 'hardware_changed',
                  active: currentFilter == 'hardware_changed',
                ),
                const SizedBox(width: 6),
                _buildFilterChip(
                  context,
                  label: 'Workspace',
                  value: 'workspace_switched',
                  active: currentFilter == 'workspace_switched',
                ),
                const SizedBox(width: 6),
                _buildFilterChip(
                  context,
                  label: 'Diagnostic',
                  value: 'diagnostic_alert',
                  active: currentFilter == 'diagnostic_alert',
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFilterChip(
    BuildContext context, {
    required String label,
    required String value,
    required bool active,
  }) {
    return ChoiceChip(
      key: Key('filter_chip_$value'),
      label: Text(label),
      selected: active,
      onSelected: (selected) {
        controller.setEventFilter(selected ? (value == 'all' ? null : value) : null);
      },
    );
  }

  Widget _buildEventCard(BuildContext context, EventEnvelope event) {
    final meta = _getEventMetadata(event.eventType);

    return Card(
      key: Key('event_card_${event.sequence}'),
      margin: EdgeInsets.zero,
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: meta.color.withValues(alpha: 0.15),
          foregroundColor: meta.color,
          child: Icon(meta.icon, size: 20),
        ),
        title: Row(
          children: [
            Text(
              '#${event.sequence.toString().padLeft(3, '0')}',
              style: const TextStyle(
                fontFamily: 'monospace',
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                event.eventType,
                style: const TextStyle(fontWeight: FontWeight.w600),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const SizedBox(height: 2),
            Text(
              _formatPayloadSummary(event),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 2),
            Text(
              event.timestampUtc,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: Colors.grey,
                  ),
            ),
          ],
        ),
        trailing: IconButton(
          key: Key('view_payload_button_${event.sequence}'),
          icon: const Icon(Icons.code, size: 18),
          tooltip: 'Inspect JSON Payload',
          onPressed: () => _showPayloadDialog(context, event),
        ),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.inbox_outlined, size: 48, color: Colors.grey),
            const SizedBox(height: 12),
            Text(
              'No events recorded matching criteria.',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
          ],
        ),
      ),
    );
  }

  void _showPayloadDialog(BuildContext context, EventEnvelope event) {
    const encoder = JsonEncoder.withIndent('  ');
    final formatted = encoder.convert(event.toJson());

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Event #${event.sequence} (${event.eventType})'),
        content: SingleChildScrollView(
          child: SelectableText(
            formatted,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  String _formatPayloadSummary(EventEnvelope event) {
    if (event.payload.containsKey('message')) {
      return event.payload['message'].toString();
    }
    if (event.payload.containsKey('audit_message')) {
      return event.payload['audit_message'].toString();
    }
    return event.payload.toString();
  }

  _EventMeta _getEventMetadata(String type) {
    switch (type) {
      case 'privacy_policy_updated':
        return const _EventMeta(Icons.shield, Colors.blue);
      case 'hardware_changed':
        return const _EventMeta(Icons.memory, Colors.purple);
      case 'workspace_switched':
        return const _EventMeta(Icons.folder_shared, Colors.amber);
      case 'diagnostic_alert':
        return const _EventMeta(Icons.sensors, Colors.teal);
      case 'adapter_health_changed':
        return const _EventMeta(Icons.health_and_safety, Colors.green);
      default:
        return const _EventMeta(Icons.info_outline, Colors.blueGrey);
    }
  }
}

class _EventMeta {
  const _EventMeta(this.icon, this.color);
  final IconData icon;
  final Color color;
}
