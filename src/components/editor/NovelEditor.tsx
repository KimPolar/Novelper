import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Underline from "@tiptap/extension-underline";
import Placeholder from "@tiptap/extension-placeholder";
import TextAlign from "@tiptap/extension-text-align";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Bold,
  Crosshair,
  Heading2,
  Italic,
  Maximize2,
  Minimize2,
  Redo2,
  Strikethrough,
  Underline as UnderlineIcon,
  Undo2,
} from "lucide-react";
import {
  NovelPreset,
  ParagraphWithId,
} from "@/components/editor/extensions/ParagraphWithId";
import { DocumentDetail } from "@/lib/tauri";
import { useEditorUiStore } from "@/stores/editorUiStore";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

interface NovelEditorProps {
  document: DocumentDetail;
  readOnly?: boolean;
}

export function NovelEditor({ document, readOnly = false }: NovelEditorProps) {
  const { t } = useTranslation();
  const { saveDocument } = useProjectStore();
  const { focusMode, typewriterMode, toggleFocusMode, toggleTypewriterMode } =
    useEditorUiStore();

  const [title, setTitle] = useState(document.title);
  const [status, setStatus] = useState(document.status);
  const saveTimerRef = useRef<number | null>(null);
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    setTitle(document.title);
    setStatus(document.status);
  }, [document.id, document.title, document.status]);

  const parsedContent = (() => {
    try {
      return JSON.parse(document.contentJson);
    } catch {
      return { type: "doc", content: [{ type: "paragraph" }] };
    }
  })();

  const editor = useEditor(
    {
      editable: !readOnly,
      extensions: [
        StarterKit.configure({
          paragraph: false,
        }),
        ParagraphWithId,
        Underline,
        Placeholder.configure({
          placeholder: t("editor.placeholder"),
        }),
        TextAlign.configure({
          types: ["heading", "paragraph"],
        }),
      ],
      content: parsedContent,
      onUpdate: ({ editor: ed }) => {
        if (readOnly) return;
        if (saveTimerRef.current) {
          window.clearTimeout(saveTimerRef.current);
        }
        saveTimerRef.current = window.setTimeout(() => {
          saveDocument(document.id, {
            contentJson: JSON.stringify(ed.getJSON()),
            plainText: ed.getText({ blockSeparator: "\n" }),
          });
        }, 700);
      },
      onSelectionUpdate: ({ editor: ed }) => {
        if (!typewriterMode || !scrollContainerRef.current) return;
        const { from } = ed.state.selection;
        const coords = ed.view.coordsAtPos(from);
        const container = scrollContainerRef.current;
        const rect = container.getBoundingClientRect();
        const targetOffset = coords.top - rect.top - rect.height / 2;
        container.scrollBy({ top: targetOffset, behavior: "smooth" });
      },
    },
    [document.id, readOnly]
  );

  useEffect(() => {
    return () => {
      if (saveTimerRef.current) {
        window.clearTimeout(saveTimerRef.current);
      }
    };
  }, []);

  if (!editor) return null;

  const currentPreset =
    (editor.getAttributes("paragraph").preset as NovelPreset) || "body";

  const setPreset = (preset: NovelPreset) => {
    editor.chain().focus().updateAttributes("paragraph", { preset }).run();
  };

  const handleTitleBlur = () => {
    const trimmed = title.trim();
    if (trimmed && trimmed !== document.title) {
      saveDocument(document.id, { title: trimmed });
    }
  };

  const handleStatusChange = (nextStatus: string) => {
    setStatus(nextStatus);
    saveDocument(document.id, { status: nextStatus });
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-800 bg-zinc-900/50 px-4 py-1.5">
        <div className="flex flex-wrap items-center gap-1">
          <button
            type="button"
            onClick={() => editor.chain().focus().undo().run()}
            disabled={!editor.can().undo()}
            className="rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100 disabled:opacity-30"
            title="Undo"
          >
            <Undo2 className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().redo().run()}
            disabled={!editor.can().redo()}
            className="rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100 disabled:opacity-30"
            title="Redo"
          >
            <Redo2 className="h-3.5 w-3.5" />
          </button>

          <div className="mx-1 h-4 w-px bg-zinc-800" />

          <button
            type="button"
            onClick={() => editor.chain().focus().toggleBold().run()}
            className={cn(
              "rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100",
              editor.isActive("bold") && "bg-zinc-800 text-amber-400"
            )}
          >
            <Bold className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().toggleItalic().run()}
            className={cn(
              "rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100",
              editor.isActive("italic") && "bg-zinc-800 text-amber-400"
            )}
          >
            <Italic className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().toggleUnderline().run()}
            className={cn(
              "rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100",
              editor.isActive("underline") && "bg-zinc-800 text-amber-400"
            )}
          >
            <UnderlineIcon className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().toggleStrike().run()}
            className={cn(
              "rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100",
              editor.isActive("strike") && "bg-zinc-800 text-amber-400"
            )}
          >
            <Strikethrough className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() =>
              editor.chain().focus().toggleHeading({ level: 2 }).run()
            }
            className={cn(
              "rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100",
              editor.isActive("heading", { level: 2 }) &&
                "bg-zinc-800 text-amber-400"
            )}
          >
            <Heading2 className="h-3.5 w-3.5" />
          </button>

          <div className="mx-1 h-4 w-px bg-zinc-800" />

          <button
            type="button"
            onClick={() => editor.chain().focus().setTextAlign("left").run()}
            className="rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
          >
            <AlignLeft className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().setTextAlign("center").run()}
            className="rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
          >
            <AlignCenter className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().setTextAlign("right").run()}
            className="rounded p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
          >
            <AlignRight className="h-3.5 w-3.5" />
          </button>

          <div className="mx-1 h-4 w-px bg-zinc-800" />

          <select
            value={currentPreset}
            onChange={(e) => setPreset(e.target.value as NovelPreset)}
            className="rounded border border-zinc-800 bg-zinc-900 px-2 py-1 text-xs text-zinc-300 focus:border-amber-500 focus:outline-none"
          >
            <option value="body">{t("preset.body")}</option>
            <option value="dialogue">{t("preset.dialogue")}</option>
            <option value="monologue">{t("preset.monologue")}</option>
            <option value="flashback">{t("preset.flashback")}</option>
          </select>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={toggleTypewriterMode}
            className={cn(
              "flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors",
              typewriterMode
                ? "bg-amber-500/20 text-amber-300"
                : "text-zinc-400 hover:bg-zinc-800"
            )}
          >
            <Crosshair className="h-3.5 w-3.5" />
            {t("editor.typewriterMode")}
          </button>
          <button
            type="button"
            onClick={toggleFocusMode}
            className={cn(
              "flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors",
              focusMode
                ? "bg-amber-500/20 text-amber-300"
                : "text-zinc-400 hover:bg-zinc-800"
            )}
          >
            {focusMode ? (
              <Minimize2 className="h-3.5 w-3.5" />
            ) : (
              <Maximize2 className="h-3.5 w-3.5" />
            )}
            {t("editor.focusMode")}
          </button>
        </div>
      </div>

      {/* Document Canvas */}
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto px-8 py-10"
      >
        <div className="mx-auto max-w-3xl">
          <div className="mb-6 flex items-center justify-between gap-4 border-b border-zinc-800/80 pb-3">
            <input
              type="text"
              value={title}
              disabled={readOnly}
              onChange={(e) => setTitle(e.target.value)}
              onBlur={handleTitleBlur}
              className="w-full bg-transparent text-2xl font-semibold text-zinc-100 focus:outline-none"
            />
            <select
              value={status}
              disabled={readOnly}
              onChange={(e) => handleStatusChange(e.target.value)}
              className="shrink-0 rounded border border-zinc-800 bg-zinc-900 px-2 py-1 text-xs text-zinc-300"
            >
              <option value="draft">{t("status.draft")}</option>
              <option value="writing">{t("status.writing")}</option>
              <option value="revising">{t("status.revising")}</option>
              <option value="done">{t("status.done")}</option>
            </select>
          </div>

          <EditorContent editor={editor} className="novel-editor" />
        </div>
      </div>

      {/* Footer Stats */}
      <div className="flex items-center justify-between border-t border-zinc-800 bg-zinc-900/40 px-4 py-1.5 text-xs text-zinc-500">
        <div className="flex gap-4">
          <span>
            {t("editor.charsWithSpace")}: {document.charsWithSpace}
          </span>
          <span>
            {t("editor.charsNoSpace")}: {document.charsNoSpace}
          </span>
          <span>
            {t("editor.words")}: {document.wordCount}
          </span>
        </div>
        <span>rev.{document.revision}</span>
      </div>
    </div>
  );
}
