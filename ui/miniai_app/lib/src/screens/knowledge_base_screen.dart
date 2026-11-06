import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

class KnowledgeBaseScreen extends StatefulWidget {
  const KnowledgeBaseScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<KnowledgeBaseScreen> createState() => _KnowledgeBaseScreenState();
}

class _KnowledgeBaseScreenState extends State<KnowledgeBaseScreen> {
  final TextEditingController _queryController = TextEditingController();

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  void _showCitationDialog(BuildContext context, String citationTag, String snippet) {
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        key: const Key('citation_inspector_dialog'),
        title: Row(
          children: [
            const Icon(Icons.bookmark, size: 20),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                'Citation: $citationTag',
                style: const TextStyle(fontSize: 14, fontFamily: 'monospace'),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        content: Container(
          width: 580,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Theme.of(ctx).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
          ),
          child: SingleChildScrollView(
            child: SelectableText(
              snippet,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
            ),
          ),
        ),
        actions: [
          TextButton.icon(
            key: const Key('copy_citation_button'),
            icon: const Icon(Icons.copy, size: 16),
            label: const Text('Copy Citation'),
            onPressed: () {
              Clipboard.setData(ClipboardData(text: citationTag));
              Navigator.of(ctx).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Citation tag copied to clipboard!')),
              );
            },
          ),
          FilledButton(
            key: const Key('close_citation_dialog_button'),
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final ctrl = widget.controller;
        final stats = ctrl.knowledgeStats;
        final docs = ctrl.knowledgeDocuments;
        final results = ctrl.ragSearchResults;

        return Scaffold(
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildHeaderBar(context, ctrl),
                const SizedBox(height: 16),
                _buildStatsAndSettingsCard(context, ctrl, stats),
                const SizedBox(height: 16),
                _buildDocumentInventoryCard(context, docs),
                const SizedBox(height: 16),
                _buildSearchTesterCard(context, ctrl, results),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildHeaderBar(BuildContext context, PlatformController ctrl) {
    final theme = Theme.of(context);
    final isIndexing = ctrl.isIndexing;

    return Card(
      key: const Key('knowledge_header_card'),
      elevation: 0,
      color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Wrap(
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          spacing: 12,
          runSpacing: 10,
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Local Knowledge Base & RAG Index Manager',
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  'Incremental hybrid document/code chunking, BM25 + dense search, and verifiable citations',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.outline,
                  ),
                ),
              ],
            ),
            FilledButton.icon(
              key: const Key('reindex_knowledge_button'),
              icon: isIndexing
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.refresh, size: 18),
              label: Text(isIndexing ? 'Indexing...' : 'Reindex Workspace'),
              onPressed: isIndexing
                  ? null
                  : () => ctrl.reindexKnowledgeBase(simulateTicks: true),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatsAndSettingsCard(
    BuildContext context,
    PlatformController ctrl,
    KnowledgeBaseStats stats,
  ) {
    final theme = Theme.of(context);

    return Card(
      key: const Key('knowledge_stats_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Index Health & Configuration',
              style: theme.textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 12,
              runSpacing: 8,
              children: [
                Chip(
                  key: const Key('files_indexed_badge'),
                  avatar: const Icon(Icons.description, size: 16),
                  label: Text('${stats.filesIndexed} Documents'),
                ),
                Chip(
                  key: const Key('chunks_indexed_badge'),
                  avatar: const Icon(Icons.grid_view, size: 16),
                  label: Text('${stats.chunksIndexed} Chunks'),
                ),
                Chip(
                  key: const Key('total_chars_badge'),
                  avatar: const Icon(Icons.text_snippet, size: 16),
                  label: Text('${stats.totalCharsIndexed} Characters'),
                ),
                Chip(
                  key: const Key('privacy_mode_badge'),
                  avatar: const Icon(Icons.shield, size: 16),
                  label: Text(
                    ctrl.localOnlyMode ? 'Offline Local Index' : 'Hybrid Consent Mode',
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            const Divider(height: 1),
            SwitchListTile(
              key: const Key('rag_grounding_switch'),
              contentPadding: EdgeInsets.zero,
              title: const Text('RAG Grounding in Chat Workspace'),
              subtitle: const Text(
                'Automatically retrieve knowledge chunks and inject verified source citations into chat responses',
              ),
              value: ctrl.enableRagGrounding,
              onChanged: (val) => ctrl.setEnableRagGrounding(val),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDocumentInventoryCard(
    BuildContext context,
    List<KnowledgeDocument> docs,
  ) {
    final theme = Theme.of(context);

    return Card(
      key: const Key('document_inventory_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.folder_open, color: theme.colorScheme.primary),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Indexed Document Inventory (${docs.length})',
                    style: theme.textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: DataTable(
                key: const Key('documents_data_table'),
                columns: const [
                  DataColumn(label: Text('File Path')),
                  DataColumn(label: Text('Type')),
                  DataColumn(label: Text('Size')),
                  DataColumn(label: Text('Lines')),
                  DataColumn(label: Text('SHA-256 Digest')),
                ],
                rows: docs.map((doc) {
                  final shortSha = doc.fileSha256.length > 12
                      ? '${doc.fileSha256.substring(0, 8)}...'
                      : doc.fileSha256;

                  return DataRow(
                    key: ValueKey('doc_row_${doc.filePath}'),
                    cells: [
                      DataCell(
                        Row(
                          key: Key('doc_row_${doc.filePath}'),
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              doc.docType == KnowledgeDocType.code
                                  ? Icons.code
                                  : Icons.article_outlined,
                              size: 16,
                            ),
                            const SizedBox(width: 6),
                            Text(
                              doc.filePath,
                              style: const TextStyle(
                                fontFamily: 'monospace',
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ),
                      ),
                      DataCell(Text(doc.docType.displayName)),
                      DataCell(Text(doc.formattedSize)),
                      DataCell(Text('${doc.lineCount}')),
                      DataCell(
                        Text(
                          shortSha,
                          style: const TextStyle(fontFamily: 'monospace', fontSize: 11),
                        ),
                      ),
                    ],
                  );
                }).toList(),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSearchTesterCard(
    BuildContext context,
    PlatformController ctrl,
    List<RetrievalResultItem> results,
  ) {
    final theme = Theme.of(context);

    return Card(
      key: const Key('rag_search_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.search, color: theme.colorScheme.primary),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Hybrid Retrieval Tester (BM25 + Dense + RRF)',
                    style: theme.textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('rag_query_input'),
                    controller: _queryController,
                    decoration: InputDecoration(
                      hintText: 'Enter search query (e.g. "AdmissionGuard CPU fallback")...',
                      border: const OutlineInputBorder(),
                      isDense: true,
                      prefixIcon: const Icon(Icons.manage_search),
                      suffixIcon: _queryController.text.isNotEmpty
                          ? IconButton(
                              icon: const Icon(Icons.clear),
                              onPressed: () {
                                _queryController.clear();
                                ctrl.setRagSearchQuery('');
                              },
                            )
                          : null,
                    ),
                    onSubmitted: (val) => ctrl.setRagSearchQuery(val),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton.icon(
                  key: const Key('rag_search_button'),
                  icon: const Icon(Icons.play_arrow, size: 18),
                  label: const Text('Search'),
                  onPressed: () => ctrl.setRagSearchQuery(_queryController.text),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                ActionChip(
                  key: const Key('sample_chip_arch'),
                  label: const Text('AdmissionGuard CPU fallback'),
                  onPressed: () {
                    _queryController.text = 'AdmissionGuard CPU fallback';
                    ctrl.setRagSearchQuery('AdmissionGuard CPU fallback');
                  },
                ),
                ActionChip(
                  key: const Key('sample_chip_privacy'),
                  label: const Text('LocalOnlyPrivacyMode consent'),
                  onPressed: () {
                    _queryController.text = 'LocalOnlyPrivacyMode consent';
                    ctrl.setRagSearchQuery('LocalOnlyPrivacyMode consent');
                  },
                ),
                ActionChip(
                  key: const Key('sample_chip_modalities'),
                  label: const Text('Decoupled Modality Adapters'),
                  onPressed: () {
                    _queryController.text = 'Decoupled Modality Adapters';
                    ctrl.setRagSearchQuery('Decoupled Modality Adapters');
                  },
                ),
              ],
            ),
            const SizedBox(height: 16),
            if (results.isNotEmpty) ...[
              Text(
                'Search Results (${results.length} Grounded Hits)',
                style: theme.textTheme.titleSmall?.copyWith(fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              ListView.builder(
                key: const Key('rag_search_results_view'),
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: results.length,
                itemBuilder: (ctx, idx) {
                  final item = results[idx];
                  return Card(
                    key: Key('result_card_${item.citationTag}'),
                    margin: const EdgeInsets.symmetric(vertical: 6),
                    color: theme.colorScheme.surfaceContainerLow,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Wrap(
                            spacing: 8,
                            runSpacing: 6,
                            crossAxisAlignment: WrapCrossAlignment.center,
                            children: [
                              Chip(
                                key: Key('citation_tag_${item.citationTag}'),
                                avatar: const Icon(Icons.bookmark_border, size: 14),
                                label: Text(
                                  item.citationTag,
                                  style: const TextStyle(
                                    fontFamily: 'monospace',
                                    fontSize: 11,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ),
                              Chip(
                                visualDensity: VisualDensity.compact,
                                label: Text('RRF: ${item.rrfScore.toStringAsFixed(4)}'),
                              ),
                              Chip(
                                visualDensity: VisualDensity.compact,
                                label: Text('Dense: ${item.denseScore.toStringAsFixed(2)}'),
                              ),
                              Chip(
                                visualDensity: VisualDensity.compact,
                                label: Text('Lexical: ${item.lexicalScore.toStringAsFixed(2)}'),
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          SelectableText(
                            item.excerpt,
                            style: theme.textTheme.bodyMedium?.copyWith(
                              fontFamily: 'monospace',
                            ),
                          ),
                          const SizedBox(height: 8),
                          Row(
                            mainAxisAlignment: MainAxisAlignment.end,
                            children: [
                              TextButton.icon(
                                key: Key('inspect_citation_${item.citationTag}'),
                                icon: const Icon(Icons.visibility, size: 16),
                                label: const Text('View Citation Source'),
                                onPressed: () => _showCitationDialog(
                                  context,
                                  item.citationTag,
                                  item.chunk.text,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ] else if (_queryController.text.isNotEmpty) ...[
              const Center(
                child: Padding(
                  padding: EdgeInsets.all(24),
                  child: Text('No knowledge chunks matched the query.'),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
