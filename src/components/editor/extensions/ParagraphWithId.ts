import Paragraph from "@tiptap/extension-paragraph";
import { Plugin, PluginKey } from "@tiptap/pm/state";

export type NovelPreset = "body" | "dialogue" | "monologue" | "flashback";

function generateBlockId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `blk_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 10)}`;
}

export const ParagraphWithId = Paragraph.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      blockId: {
        default: null,
        parseHTML: (element: HTMLElement) => element.getAttribute("data-block-id"),
        renderHTML: (attributes: Record<string, unknown>) => {
          if (!attributes.blockId) return {};
          return { "data-block-id": attributes.blockId };
        },
      },
      preset: {
        default: "body" as NovelPreset,
        parseHTML: (element: HTMLElement) =>
          (element.getAttribute("data-preset") as NovelPreset) || "body",
        renderHTML: (attributes: Record<string, unknown>) => {
          if (!attributes.preset || attributes.preset === "body") return {};
          return { "data-preset": attributes.preset };
        },
      },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey("paragraphBlockId"),
        appendTransaction(_transactions, _oldState, newState) {
          const seen = new Set<string>();
          let tr = newState.tr;
          let modified = false;

          newState.doc.descendants((node, pos) => {
            if (node.type.name === "paragraph") {
              const currentId = node.attrs.blockId as string | null;
              if (!currentId || seen.has(currentId)) {
                const nextId = generateBlockId();
                seen.add(nextId);
                tr = tr.setNodeMarkup(pos, undefined, {
                  ...node.attrs,
                  blockId: nextId,
                });
                modified = true;
              } else {
                seen.add(currentId);
              }
            }
          });

          return modified ? tr : null;
        },
      }),
    ];
  },
});
