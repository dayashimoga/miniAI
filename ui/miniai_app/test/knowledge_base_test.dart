import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/knowledge_base_screen.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';

void main() {
  group('Subphase 5.3.2 Knowledge Base & Citation Inspector Widget Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    testWidgets('ShellScreen navigates to Tab 7 Knowledge Base across desktop and mobile', (tester) async {
      // 1. Desktop layout (1024x768)
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('desktop_navigation_rail')), findsOneWidget);
      expect(find.byKey(const Key('nav_rail_knowledge')), findsOneWidget);

      await tester.tap(find.byKey(const Key('nav_rail_knowledge')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, 7);
      expect(find.byType(KnowledgeBaseScreen), findsOneWidget);
      expect(find.byKey(const Key('knowledge_header_card')), findsOneWidget);

      // 2. Mobile layout (400x800)
      tester.view.physicalSize = const Size(400, 800);
      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('nav_bar_knowledge')), findsOneWidget);

      // Switch to another tab first
      controller.setSelectedNavigationIndex(0);
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, 0);

      await tester.tap(find.byKey(const Key('nav_bar_knowledge')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, 7);
      expect(find.byType(KnowledgeBaseScreen), findsOneWidget);
    });

    testWidgets('KnowledgeBaseScreen renders stats badges, document inventory, and toggle', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: KnowledgeBaseScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('knowledge_header_card')), findsOneWidget);
      expect(find.byKey(const Key('knowledge_stats_card')), findsOneWidget);
      expect(find.byKey(const Key('files_indexed_badge')), findsOneWidget);
      expect(find.byKey(const Key('chunks_indexed_badge')), findsOneWidget);
      expect(find.byKey(const Key('document_inventory_card')), findsOneWidget);
      expect(find.byKey(const Key('rag_grounding_switch')), findsOneWidget);

      // Check document rows in data table
      expect(find.byKey(const Key('doc_row_ARCHITECTURE.md')), findsOneWidget);
      expect(find.byKey(const Key('doc_row_SECURITY.md')), findsOneWidget);

      // Toggle RAG grounding switch
      expect(controller.enableRagGrounding, isTrue);
      await tester.tap(find.byKey(const Key('rag_grounding_switch')));
      await tester.pumpAndSettle();
      expect(controller.enableRagGrounding, isFalse);

      await tester.tap(find.byKey(const Key('rag_grounding_switch')));
      await tester.pumpAndSettle();
      expect(controller.enableRagGrounding, isTrue);
    });

    testWidgets('Reindex button triggers reindexing and updates stats', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: KnowledgeBaseScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      final reindexBtn = find.byKey(const Key('reindex_knowledge_button'));
      expect(reindexBtn, findsOneWidget);

      await tester.tap(reindexBtn);
      await tester.pump(const Duration(milliseconds: 30));
      await tester.pumpAndSettle();

      expect(controller.knowledgeStats.filesIndexed, 3);
      expect(controller.knowledgeStats.chunksIndexed, 4);
    });

    testWidgets('Interactive search query and sample chips filter results and open citation dialog', (tester) async {
      tester.view.physicalSize = const Size(1024, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: KnowledgeBaseScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Tap sample chip for AdmissionGuard
      final sampleChip = find.byKey(const Key('sample_chip_arch'));
      expect(sampleChip, findsOneWidget);
      await tester.tap(sampleChip);
      await tester.pumpAndSettle();

      expect(controller.ragSearchResults, isNotEmpty);
      expect(find.byKey(const Key('rag_search_results_view')), findsOneWidget);
      expect(find.byKey(const Key('citation_tag_ARCHITECTURE.md#L1-L24')), findsOneWidget);

      // Tap inspect citation button
      final inspectBtn = find.byKey(const Key('inspect_citation_ARCHITECTURE.md#L1-L24'));
      expect(inspectBtn, findsOneWidget);
      await tester.ensureVisible(inspectBtn);
      await tester.pumpAndSettle();
      await tester.tap(inspectBtn);
      await tester.pumpAndSettle();

      // Verify citation inspector dialog
      expect(find.byKey(const Key('citation_inspector_dialog')), findsOneWidget);
      expect(
        find.descendant(
          of: find.byKey(const Key('citation_inspector_dialog')),
          matching: find.textContaining('MiniAI System Architecture'),
        ),
        findsOneWidget,
      );

      // Tap close
      await tester.tap(find.byKey(const Key('close_citation_dialog_button')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('citation_inspector_dialog')), findsNothing);
    });

    testWidgets('ChatWorkspaceScreen displays citation chips when RAG grounding attaches sources', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      controller.setSelectedNavigationIndex(5); // Chat tab

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Send a query matching architecture knowledge
      await controller.sendMessage(
        'How does AdmissionGuard work with CPU fallback?',
        delayTokens: false,
      );
      await tester.pumpAndSettle();

      final currentMsgs = controller.currentMessages;
      expect(currentMsgs.length, greaterThanOrEqualTo(2));
      final lastMsg = currentMsgs.last;
      expect(lastMsg.citations, isNotEmpty);
      expect(lastMsg.citations, contains('ARCHITECTURE.md#L1-L24'));

      // Find citation chip on assistant message bubble
      final citationChip = find.byKey(const Key('citation_chip_ARCHITECTURE.md#L1-L24'));
      expect(citationChip, findsOneWidget);

      // Tap citation chip to inspect source modal
      await tester.scrollUntilVisible(
        citationChip,
        50.0,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(citationChip);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('chat_citation_dialog')), findsOneWidget);
      expect(find.textContaining('Source: ARCHITECTURE.md#L1-L24'), findsOneWidget);

      // Tap close button
      await tester.tap(find.byKey(const Key('close_chat_citation_dialog_button')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('chat_citation_dialog')), findsNothing);
    });

    test('Stage 5 data contracts round-trip serialize to JSON and format sizes', () {
      const doc = KnowledgeDocument(
        filePath: 'test/path.rs',
        fileSha256: 'abc123sha',
        sizeBytes: 2048 * 1024,
        lineCount: 150,
        docType: KnowledgeDocType.code,
        lastModifiedEpochSecs: 1700000000,
      );
      expect(doc.formattedSize, '2.0 MB');
      const docBytes = KnowledgeDocument(
        filePath: 'test/small.txt',
        fileSha256: 'sha',
        sizeBytes: 512,
        lineCount: 10,
        docType: KnowledgeDocType.markdown,
        lastModifiedEpochSecs: 1700000000,
      );
      expect(docBytes.formattedSize, '512 B');

      final docJson = doc.toJson();
      final roundTripDoc = KnowledgeDocument.fromJson(docJson);
      expect(roundTripDoc.filePath, 'test/path.rs');
      expect(roundTripDoc.sizeBytes, 2048 * 1024);

      const chunk = KnowledgeChunk(
        chunkId: 'chunk_1',
        filePath: 'test/path.rs',
        lineStart: 1,
        lineEnd: 20,
        text: 'fn main() {}',
        citationTag: 'test/path.rs#L1-L20',
        tokenEstimate: 5,
        docType: KnowledgeDocType.code,
      );
      final chunkJson = chunk.toJson();
      final roundTripChunk = KnowledgeChunk.fromJson(chunkJson);
      expect(roundTripChunk.chunkId, 'chunk_1');
      expect(roundTripChunk.citationTag, 'test/path.rs#L1-L20');

      const resultItem = RetrievalResultItem(
        chunk: chunk,
        rrfScore: 0.033,
        denseScore: 0.85,
        lexicalScore: 4.2,
        citationTag: 'test/path.rs#L1-L20',
        excerpt: 'fn main() {}',
      );
      final itemJson = resultItem.toJson();
      final roundTripItem = RetrievalResultItem.fromJson(itemJson);
      expect(roundTripItem.rrfScore, 0.033);
      expect(roundTripItem.chunk.filePath, 'test/path.rs');

      const tool = ToolDescriptorContract(
        name: 'workspace_reader',
        description: 'Read workspace files',
        requiredCapability: ToolCapabilityContract.workspaceRead,
      );
      final toolJson = tool.toJson();
      final roundTripTool = ToolDescriptorContract.fromJson(toolJson);
      expect(roundTripTool.name, 'workspace_reader');
      expect(roundTripTool.requiredCapability, ToolCapabilityContract.workspaceRead);
      expect(ToolCapabilityContract.fromWire('unknown'), ToolCapabilityContract.compute);

      const stats = KnowledgeBaseStats(
        filesIndexed: 5,
        chunksIndexed: 12,
        totalCharsIndexed: 3000,
        lastIndexedAtEpochSecs: 1700000000,
      );
      final statsJson = stats.toJson();
      final roundTripStats = KnowledgeBaseStats.fromJson(statsJson);
      expect(roundTripStats.filesIndexed, 5);
      expect(roundTripStats.chunksIndexed, 12);
    });
  });
}
