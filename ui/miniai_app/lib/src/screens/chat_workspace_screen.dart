import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class ChatWorkspaceScreen extends StatefulWidget {
  const ChatWorkspaceScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<ChatWorkspaceScreen> createState() => _ChatWorkspaceScreenState();
}

class _ChatWorkspaceScreenState extends State<ChatWorkspaceScreen> {
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _handleSend() {
    final text = _inputController.text;
    if (text.trim().isEmpty || widget.controller.isGenerating) return;
    _inputController.clear();
    widget.controller.sendMessage(text);
    _scrollToBottom();
  }

  void _showRenameDialog(BuildContext context, ChatSession session) {
    final renameController = TextEditingController(text: session.title);
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Rename Conversation'),
        content: TextField(
          key: const Key('rename_session_input'),
          controller: renameController,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Conversation Title',
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            key: const Key('confirm_rename_session_button'),
            onPressed: () {
              widget.controller.renameChatSession(
                session.id,
                renameController.text,
              );
              Navigator.of(ctx).pop();
            },
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _showEditMessageDialog(BuildContext context, ChatMessage message) {
    final editController = TextEditingController(text: message.content);
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Edit Message'),
        content: TextField(
          key: const Key('edit_message_input'),
          controller: editController,
          maxLines: 4,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Message Content',
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            key: const Key('confirm_edit_message_button'),
            onPressed: () {
              widget.controller.editMessage(
                message.id,
                editController.text,
              );
              Navigator.of(ctx).pop();
            },
            child: const Text('Save Changes'),
          ),
        ],
      ),
    );
  }

  void _showTuningDialog(BuildContext context) {
    double temp = widget.controller.chatTemperature;
    int maxTok = widget.controller.chatMaxTokens;

    showDialog<void>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: const Text('Inference Hyperparameters'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Temperature: ${temp.toStringAsFixed(2)}'),
              Slider(
                key: const Key('temperature_slider'),
                value: temp,
                min: 0.0,
                max: 2.0,
                divisions: 20,
                label: temp.toStringAsFixed(2),
                onChanged: (val) {
                  setDialogState(() => temp = val);
                  widget.controller.setChatTemperature(val);
                },
              ),
              const SizedBox(height: 12),
              Text('Max Output Tokens: $maxTok'),
              Slider(
                key: const Key('max_tokens_slider'),
                value: maxTok.toDouble(),
                min: 32,
                max: 2048,
                divisions: 63,
                label: maxTok.toString(),
                onChanged: (val) {
                  setDialogState(() => maxTok = val.round());
                  widget.controller.setChatMaxTokens(val.round());
                },
              ),
            ],
          ),
          actions: [
            FilledButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Done'),
            ),
          ],
        ),
      ),
    );
  }

  void _showCitationDialog(BuildContext context, String citationTag) {
    final snippet = widget.controller.getCitationSnippet(citationTag);
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        key: const Key('chat_citation_dialog'),
        title: Row(
          children: [
            const Icon(Icons.bookmark, size: 20),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                'Source: $citationTag',
                style: const TextStyle(fontSize: 14, fontFamily: 'monospace'),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        content: Container(
          constraints: const BoxConstraints(maxWidth: 520),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Theme.of(ctx).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
          ),
          child: SingleChildScrollView(
            child: Text(
              snippet,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
            ),
          ),
        ),
        actions: [
          TextButton.icon(
            key: const Key('copy_chat_citation_button'),
            icon: const Icon(Icons.copy, size: 16),
            label: const Text('Copy Citation Tag'),
            onPressed: () {
              Clipboard.setData(ClipboardData(text: citationTag));
              Navigator.of(ctx).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Citation copied to clipboard!')),
              );
            },
          ),
          FilledButton(
            key: const Key('close_chat_citation_dialog_button'),
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildFormattedMessageContent(BuildContext context, ChatMessage message) {
    final theme = Theme.of(context);
    final content = message.content;

    String? thinkContent;
    String mainContent = content;

    final thinkStart = content.indexOf('<think>');
    final thinkEnd = content.indexOf('</think>');
    if (thinkStart != -1 && thinkEnd != -1 && thinkEnd > thinkStart) {
      thinkContent = content.substring(thinkStart + 7, thinkEnd).trim();
      mainContent = (content.substring(0, thinkStart) + content.substring(thinkEnd + 8)).trim();
    }

    final children = <Widget>[];

    if (thinkContent != null && thinkContent.isNotEmpty) {
      children.add(
        Container(
          key: Key('think_block_${message.id}'),
          margin: const EdgeInsets.only(bottom: 8),
          decoration: BoxDecoration(
            color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.4),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: theme.colorScheme.outlineVariant.withValues(alpha: 0.5),
            ),
          ),
          child: ExpansionTile(
            key: Key('think_tile_${message.id}'),
            dense: true,
            tilePadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 0),
            childrenPadding: const EdgeInsets.all(10),
            leading: const Icon(Icons.psychology_outlined, size: 16),
            title: const Text(
              'Reasoning Process',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, fontStyle: FontStyle.italic),
            ),
            children: [
              Text(
                thinkContent,
                style: theme.textTheme.bodySmall?.copyWith(fontFamily: 'monospace', fontSize: 11),
              ),
            ],
          ),
        ),
      );
    }

    if (!mainContent.contains('```')) {
      children.add(
        Text(
          mainContent,
          style: theme.textTheme.bodyMedium,
        ),
      );
      return children.length == 1
          ? children.first
          : Column(crossAxisAlignment: CrossAxisAlignment.start, children: children);
    }

    final parts = mainContent.split('```');
    final widgets = <Widget>[...children];

    for (var i = 0; i < parts.length; i++) {
      final part = parts[i];
      if (part.isEmpty) continue;

      if (i % 2 == 1) {
        // Code block
        var language = '';
        var code = part;
        final firstNewline = part.indexOf('\n');
        if (firstNewline != -1) {
          language = part.substring(0, firstNewline).trim();
          code = part.substring(firstNewline + 1);
        }

        widgets.add(
          Container(
            key: Key('code_block_${message.id}_$i'),
            margin: const EdgeInsets.symmetric(vertical: 6),
            decoration: BoxDecoration(
              color: theme.colorScheme.surfaceContainerLowest,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: theme.colorScheme.outlineVariant.withValues(alpha: 0.5),
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: theme.colorScheme.surfaceContainer,
                    borderRadius: const BorderRadius.vertical(top: Radius.circular(7)),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        language.isNotEmpty ? language : 'code',
                        style: theme.textTheme.labelSmall?.copyWith(fontFamily: 'monospace'),
                      ),
                      InkWell(
                        key: Key('copy_code_${message.id}_$i'),
                        onTap: () {
                          Clipboard.setData(ClipboardData(text: code));
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('Code copied to clipboard!')),
                          );
                        },
                        child: Row(
                          children: [
                            const Icon(Icons.copy, size: 12),
                            const SizedBox(width: 4),
                            Text('Copy', style: theme.textTheme.labelSmall),
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(10),
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: Text(
                      code.trimRight(),
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12.5,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      } else {
        // Prose block
        widgets.add(
          Text(
            part,
            style: theme.textTheme.bodyMedium,
          ),
        );
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: widgets,
    );
  }

  bool _isModelUnverified(String modelId) {
    final status = widget.controller.getModelStatus(modelId);
    return status.status != DownloadStatus.verified;
  }

  Widget _buildModelReadinessBanner(BuildContext context) {
    final selectedId = widget.controller.selectedChatModelId;
    final manifest = widget.controller.models
        .where((m) => m.modelId == selectedId)
        .firstOrNull;
    final modelName = manifest?.name ?? selectedId;

    return Container(
      key: const Key('model_readiness_banner'),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Colors.amber.withValues(alpha: 0.15),
      child: Row(
        children: [
          const Icon(Icons.info_outline, color: Colors.orange, size: 18),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              'Model "$modelName" is pending download & verification in local cache.',
              style: const TextStyle(fontSize: 12),
            ),
          ),
          FilledButton.tonal(
            key: Key('ready_model_button_$selectedId'),
            onPressed: () {
              widget.controller.completeDownload(selectedId);
            },
            child: const Text('Verify & Ready', style: TextStyle(fontSize: 11)),
          ),
        ],
      ),
    );
  }

  void _showMemoryDialog(BuildContext context) {
    showDialog<void>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) {
          final memories = widget.controller.memories;
          return AlertDialog(
            key: const Key('memory_inspector_dialog'),
            title: const Row(
              children: [
                Icon(Icons.psychology, size: 22),
                SizedBox(width: 8),
                Text('3-Tier Cognitive Memory'),
              ],
            ),
            content: SizedBox(
              width: 520,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Durable semantic facts and user preferences remembered across conversation sessions under strict local-only privacy guarantees.',
                    style: TextStyle(fontSize: 12),
                  ),
                  const SizedBox(height: 12),
                  if (memories.isEmpty)
                    const Padding(
                      padding: EdgeInsets.symmetric(vertical: 24),
                      child: Center(child: Text('No memories stored yet.')),
                    )
                  else
                    ConstrainedBox(
                      constraints: const BoxConstraints(maxHeight: 260),
                      child: ListView.separated(
                        shrinkWrap: true,
                        itemCount: memories.length,
                        separatorBuilder: (_, __) => const Divider(height: 1),
                        itemBuilder: (ctx, i) {
                          final mem = memories[i];
                          return ListTile(
                            key: Key('memory_item_${mem.id}'),
                            dense: true,
                            contentPadding: EdgeInsets.zero,
                            title: Text(mem.content,
                                style: const TextStyle(fontSize: 12.5)),
                            subtitle: Text(
                              'Type: ${mem.type} • Confidence: ${(mem.confidence * 100).toInt()}%',
                              style: const TextStyle(fontSize: 10.5),
                            ),
                            trailing: IconButton(
                              key: Key('delete_memory_${mem.id}'),
                              icon: const Icon(Icons.delete_outline, size: 16),
                              onPressed: () {
                                widget.controller.deleteMemory(mem.id);
                                setDialogState(() {});
                              },
                            ),
                          );
                        },
                      ),
                    ),
                ],
              ),
            ),
            actions: [
              if (memories.isNotEmpty)
                TextButton.icon(
                  key: const Key('clear_memory_button'),
                  icon: const Icon(Icons.delete_sweep, size: 16),
                  label: const Text('Clear All Memories'),
                  onPressed: () {
                    widget.controller.clearMemories();
                    setDialogState(() {});
                  },
                ),
              FilledButton(
                key: const Key('close_memory_dialog_button'),
                onPressed: () => Navigator.of(ctx).pop(),
                child: const Text('Done'),
              ),
            ],
          );
        },
      ),
    );
  }

  void _showExportDialog(BuildContext context) {
    final activeId = widget.controller.activeSessionId;
    if (activeId == null) return;

    showModalBottomSheet<void>(
      context: context,
      builder: (ctx) => SafeArea(
        child: Wrap(
          children: [
            ListTile(
              leading: const Icon(Icons.code),
              title: const Text('Export as Markdown (.md)'),
              onTap: () {
                Navigator.of(ctx).pop();
                final md = widget.controller.exportSessionMarkdown(activeId);
                Clipboard.setData(ClipboardData(text: md));
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Markdown copied to clipboard!'),
                  ),
                );
              },
            ),
            ListTile(
              leading: const Icon(Icons.data_object),
              title: const Text('Export as JSON (.json)'),
              onTap: () {
                Navigator.of(ctx).pop();
                final json = widget.controller.exportSessionJson(activeId);
                Clipboard.setData(ClipboardData(text: json));
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('JSON copied to clipboard!'),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  void _showModelSwitcher(BuildContext context) {
    final availableModels = widget.controller.models
        .where((m) => m.modality == Modality.llm)
        .toList();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Select Neural Model'),
        content: SizedBox(
          width: 320,
          child: ListView.builder(
            shrinkWrap: true,
            itemCount: availableModels.length,
            itemBuilder: (c, idx) {
              final m = availableModels[idx];
              final isSelected = m.modelId == widget.controller.selectedChatModelId;
              return ListTile(
                selected: isSelected,
                leading: Icon(isSelected ? Icons.check_circle : Icons.circle_outlined),
                title: Text(m.name),
                subtitle: Text('${m.quantization} • ${m.parameterCount}'),
                onTap: () {
                  widget.controller.setSelectedChatModel(m.modelId);
                  Navigator.of(ctx).pop();
                },
              );
            },
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return LayoutBuilder(
          builder: (context, constraints) {
            final isWide = constraints.maxWidth >= 760;

            return CallbackShortcuts(
              bindings: {
                const SingleActivator(LogicalKeyboardKey.keyN, control: true): () {
                  widget.controller.createChatSession();
                },
                const SingleActivator(LogicalKeyboardKey.keyN, meta: true): () {
                  widget.controller.createChatSession();
                },
                const SingleActivator(LogicalKeyboardKey.keyK, control: true): () {
                  _showModelSwitcher(context);
                },
                const SingleActivator(LogicalKeyboardKey.keyK, meta: true): () {
                  _showModelSwitcher(context);
                },
              },
              child: Focus(
                autofocus: true,
                child: Scaffold(
                  drawer: isWide ? null : Drawer(child: _buildSessionSidebar(context)),
                  body: Row(
                    children: [
                      if (isWide) ...[
                        SizedBox(
                          width: 280,
                          child: _buildSessionSidebar(context),
                        ),
                        const VerticalDivider(width: 1, thickness: 1),
                      ],
                      Expanded(
                        child: Column(
                          children: [
                            _buildTopToolbar(context, isWide),
                            const Divider(height: 1, thickness: 1),
                            if (_isModelUnverified(widget.controller.selectedChatModelId))
                              _buildModelReadinessBanner(context),
                            Expanded(child: _buildMessageTimeline(context)),
                            if (widget.controller.isGenerating)
                              _buildStreamingStatusBar(context),
                            _buildInputComposer(context),
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            );
          },
        );
      },
    );
  }

  Widget _buildTopToolbar(BuildContext context, bool isWide) {
    final activeSession = widget.controller.activeSession;
    final availableModels = widget.controller.models
        .where((m) => m.modality == Modality.llm)
        .toList();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
      child: Row(
        children: [
          if (!isWide)
            Builder(
              builder: (scaffoldCtx) => IconButton(
                key: const Key('toggle_sessions_button'),
                tooltip: 'Toggle Conversations Drawer',
                visualDensity: VisualDensity.compact,
                icon: const Icon(Icons.menu),
                onPressed: () {
                  Scaffold.of(scaffoldCtx).openDrawer();
                },
              ),
            ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  activeSession?.title ?? 'No Active Chat',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                Text(
                  'Offline-first • ${widget.controller.activeExecutionBackend.displayName}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.primary,
                        fontSize: 10,
                      ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
          const SizedBox(width: 4),
          // Model Selector Dropdown
          ConstrainedBox(
            constraints: BoxConstraints(maxWidth: isWide ? 220 : 130),
            child: DropdownButton<String>(
              key: const Key('chat_model_selector'),
              isDense: true,
              isExpanded: true,
              value: availableModels.any(
                      (m) => m.modelId == widget.controller.selectedChatModelId)
                  ? widget.controller.selectedChatModelId
                  : (availableModels.isNotEmpty
                      ? availableModels.first.modelId
                      : null),
              items: availableModels.map((m) {
                return DropdownMenuItem<String>(
                  value: m.modelId,
                  child: Text(
                    isWide ? m.name : m.modelId,
                    style: const TextStyle(fontSize: 11),
                    overflow: TextOverflow.ellipsis,
                  ),
                );
              }).toList(),
              onChanged: (selected) {
                if (selected != null) {
                  widget.controller.setSelectedChatModel(selected);
                }
              },
            ),
          ),
          IconButton(
            key: const Key('tuning_settings_button'),
            tooltip: 'Tuning Parameters',
            visualDensity: VisualDensity.compact,
            icon: const Icon(Icons.tune, size: 18),
            onPressed: () => _showTuningDialog(context),
          ),
          IconButton(
            key: const Key('memory_inspector_button'),
            tooltip: 'Cognitive Memory Inspector',
            visualDensity: VisualDensity.compact,
            icon: const Icon(Icons.psychology_outlined, size: 18),
            onPressed: () => _showMemoryDialog(context),
          ),
          IconButton(
            key: const Key('export_chat_button'),
            tooltip: 'Export Conversation',
            visualDensity: VisualDensity.compact,
            icon: const Icon(Icons.download, size: 18),
            onPressed: () => _showExportDialog(context),
          ),
        ],
      ),
    );
  }

  Widget _buildSessionSidebar(BuildContext context) {
    final sessions = widget.controller.chatSessions;
    final activeId = widget.controller.activeSessionId;

    return Container(
      color: Theme.of(context).colorScheme.surfaceContainerLow,
      child: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                children: [
                  Text(
                    'Conversations',
                    style: Theme.of(context).textTheme.titleSmall?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                  const Spacer(),
                  IconButton.filledTonal(
                    key: const Key('new_chat_button'),
                    tooltip: 'New Conversation',
                    visualDensity: VisualDensity.compact,
                    icon: const Icon(Icons.add, size: 18),
                    onPressed: () {
                      widget.controller.createChatSession();
                      final scaffold = Scaffold.maybeOf(context);
                      if (scaffold != null && scaffold.isDrawerOpen) {
                        Navigator.of(context).pop();
                      }
                    },
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: sessions.isEmpty
                  ? const Center(
                      child: Text('No sessions yet'),
                    )
                  : ListView.builder(
                      itemCount: sessions.length,
                      itemBuilder: (ctx, index) {
                        final session = sessions[index];
                        final isSelected = session.id == activeId;

                        return ListTile(
                          key: Key('session_item_${session.id}'),
                          selected: isSelected,
                          dense: true,
                          selectedTileColor: Theme.of(context)
                              .colorScheme
                              .primaryContainer
                              .withValues(alpha: 0.4),
                          leading: Icon(
                            isSelected
                                ? Icons.chat_bubble
                                : Icons.chat_bubble_outline,
                            size: 16,
                          ),
                          title: Text(
                            session.title,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(
                              fontWeight: isSelected
                                  ? FontWeight.bold
                                  : FontWeight.normal,
                              fontSize: 12,
                            ),
                          ),
                          trailing: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              IconButton(
                                key: Key('rename_session_${session.id}'),
                                icon: const Icon(Icons.edit, size: 15),
                                visualDensity: VisualDensity.compact,
                                tooltip: 'Rename Chat',
                                padding: EdgeInsets.zero,
                                constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                                onPressed: () =>
                                    _showRenameDialog(context, session),
                              ),
                              IconButton(
                                key: Key('delete_session_${session.id}'),
                                icon: const Icon(Icons.delete_outline, size: 15),
                                visualDensity: VisualDensity.compact,
                                tooltip: 'Delete Chat',
                                padding: EdgeInsets.zero,
                                constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                                onPressed: () =>
                                    widget.controller.deleteChatSession(session.id),
                              ),
                            ],
                          ),
                          onTap: () {
                            widget.controller.selectChatSession(session.id);
                            final scaffold = Scaffold.maybeOf(context);
                            if (scaffold != null && scaffold.isDrawerOpen) {
                              Navigator.of(context).pop();
                            }
                          },
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMessageTimeline(BuildContext context) {
    final messages = widget.controller.currentMessages;

    if (messages.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.forum_outlined,
              size: 56,
              color: Theme.of(context).colorScheme.outline,
            ),
            const SizedBox(height: 12),
            Text(
              'No messages in this conversation yet.',
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    color: Theme.of(context).colorScheme.outline,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              'Type a message below to start local inference.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      );
    }

    return SelectionArea(
      child: ListView.builder(
        key: const Key('chat_message_list'),
        controller: _scrollController,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        itemCount: messages.length,
        itemBuilder: (ctx, index) {
          final message = messages[index];
          return _buildMessageBubble(context, message);
        },
      ),
    );
  }

  Widget _buildMessageBubble(BuildContext context, ChatMessage message) {
    final isUser = message.role == ChatMessageRole.user;
    final isAssistant = message.role == ChatMessageRole.assistant;

    final theme = Theme.of(context);
    final bubbleColor = isUser
        ? theme.colorScheme.primaryContainer
        : isAssistant
            ? theme.colorScheme.surfaceContainerHigh
            : theme.colorScheme.tertiaryContainer;

    return Padding(
      key: Key('chat_message_${message.id}'),
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment:
            isUser ? MainAxisAlignment.end : MainAxisAlignment.start,
        children: [
          if (!isUser) ...[
            CircleAvatar(
              radius: 14,
              backgroundColor: theme.colorScheme.secondaryContainer,
              child: Icon(
                isAssistant ? Icons.smart_toy : Icons.settings,
                size: 16,
                color: theme.colorScheme.onSecondaryContainer,
              ),
            ),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Container(
              constraints: const BoxConstraints(maxWidth: 680),
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: bubbleColor,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Text(
                        message.role.displayName,
                        style: theme.textTheme.labelMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      if (message.modelId != null) ...[
                        const SizedBox(width: 6),
                        Flexible(
                          child: Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: theme.colorScheme.surface,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              message.modelId!,
                              style: theme.textTheme.labelSmall,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ),
                      ],
                      const Spacer(),
                      Container(
                        key: Key('token_badge_${message.id}'),
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: theme.colorScheme.surfaceContainerHighest,
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Text(
                          '${message.tokenCount} tok',
                          style: const TextStyle(fontSize: 10),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 6),
                  _buildFormattedMessageContent(context, message),
                  if (message.citations.isNotEmpty) ...[
                    const SizedBox(height: 8),
                    Wrap(
                      spacing: 6,
                      runSpacing: 6,
                      children: message.citations.map((citation) {
                        return ActionChip(
                          key: Key('citation_chip_$citation'),
                          avatar: const Icon(Icons.bookmark_border, size: 14),
                          label: Text(
                            citation,
                            style: const TextStyle(
                              fontSize: 11,
                              fontFamily: 'monospace',
                            ),
                          ),
                          onPressed: () => _showCitationDialog(context, citation),
                        );
                      }).toList(),
                    ),
                  ],
                  const SizedBox(height: 6),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      if (isAssistant)
                        IconButton(
                          key: Key('copy_message_${message.id}'),
                          icon: const Icon(Icons.copy, size: 16),
                          visualDensity: VisualDensity.compact,
                          tooltip: 'Copy Message',
                          onPressed: () {
                            Clipboard.setData(
                              ClipboardData(text: message.content),
                            );
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('Copied to clipboard!'),
                              ),
                            );
                          },
                        ),
                      if (isUser)
                        IconButton(
                          key: Key('edit_message_${message.id}'),
                          icon: const Icon(Icons.edit, size: 16),
                          visualDensity: VisualDensity.compact,
                          tooltip: 'Edit Message',
                          onPressed: () =>
                              _showEditMessageDialog(context, message),
                        ),
                      IconButton(
                        key: Key('delete_message_${message.id}'),
                        icon: const Icon(Icons.delete_outline, size: 16),
                        visualDensity: VisualDensity.compact,
                        tooltip: 'Delete Message',
                        onPressed: () =>
                            widget.controller.deleteMessage(message.id),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          if (isUser) ...[
            const SizedBox(width: 8),
            CircleAvatar(
              radius: 14,
              backgroundColor: theme.colorScheme.primary,
              child: Icon(
                Icons.person,
                size: 16,
                color: theme.colorScheme.onPrimary,
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildStreamingStatusBar(BuildContext context) {
    final theme = Theme.of(context);
    final tokPerSec = widget.controller.generationTokensPerSecond;
    final tokenCount = widget.controller.generationTokenCount;
    final elapsed = widget.controller.generationElapsedSeconds;

    return Container(
      key: const Key('generation_progress_card'),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: theme.colorScheme.secondaryContainer.withValues(alpha: 0.5),
      child: Column(
        children: [
          const LinearProgressIndicator(minHeight: 2),
          const SizedBox(height: 6),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                Chip(
                  key: const Key('tokens_per_second_badge'),
                  avatar: const Icon(Icons.speed, size: 14),
                  visualDensity: VisualDensity.compact,
                  label: Text(
                    '${tokPerSec.toStringAsFixed(1)} tok/s',
                    style: const TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Chip(
                  key: const Key('token_count_badge'),
                  avatar: const Icon(Icons.numbers, size: 14),
                  visualDensity: VisualDensity.compact,
                  label: Text(
                    '$tokenCount tokens',
                    style: const TextStyle(fontSize: 11),
                  ),
                ),
                const SizedBox(width: 8),
                Chip(
                  key: const Key('generation_elapsed_badge'),
                  avatar: const Icon(Icons.timer_outlined, size: 14),
                  visualDensity: VisualDensity.compact,
                  label: Text(
                    '${elapsed.toStringAsFixed(1)}s',
                    style: const TextStyle(fontSize: 11),
                  ),
                ),
                const SizedBox(width: 16),
                ElevatedButton.icon(
                  key: const Key('cancel_generation_button'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: theme.colorScheme.errorContainer,
                    foregroundColor: theme.colorScheme.onErrorContainer,
                    visualDensity: VisualDensity.compact,
                  ),
                  icon: const Icon(Icons.stop, size: 16),
                  label: const Text('Stop Generation'),
                  onPressed: () => widget.controller.cancelGeneration(),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInputComposer(BuildContext context) {
    final isGenerating = widget.controller.isGenerating;

    return Container(
      key: const Key('chat_composer_bar'),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      color: Theme.of(context).colorScheme.surface,
      child: Row(
        children: [
          Expanded(
            child: TextField(
              key: const Key('chat_input_field'),
              controller: _inputController,
              enabled: !isGenerating,
              minLines: 1,
              maxLines: 4,
              textInputAction: TextInputAction.send,
              onSubmitted: (_) => _handleSend(),
              decoration: InputDecoration(
                hintText: isGenerating
                    ? 'Generating offline response...'
                    : 'Ask anything (local CPU inference)...',
                border: const OutlineInputBorder(
                  borderRadius: BorderRadius.all(Radius.circular(24)),
                ),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 12,
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),
          IconButton.filled(
            key: const Key('send_message_button'),
            tooltip: 'Send Message',
            icon: const Icon(Icons.send),
            onPressed: isGenerating ? null : _handleSend,
          ),
        ],
      ),
    );
  }
}
