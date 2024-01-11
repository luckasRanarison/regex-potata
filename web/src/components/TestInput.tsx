import ReactCodeMirror, {
  Decoration,
  Extension,
  MatchDecorator,
  ViewPlugin,
} from "@uiw/react-codemirror";
import { useEffect, useState } from "react";

const TestInput = ({ pattern }: { pattern: string }) => {
  const [testInput, setTestInput] = useState("");
  const [highlightExtension, setHighlightExtension] = useState<Extension>();

  useEffect(() => {
    if (!pattern) {
      return;
    }

    try {
      const decoration = Decoration.mark({
        class: "highlight-chunk",
        inclusiveStart: true,
        inclusiveEnd: false,
      });
      const matchDecorator = new MatchDecorator({
        regexp: new RegExp(pattern, "g"), // TODO: use potata
        decoration,
      });

      const plugin = ViewPlugin.define(
        (view) => ({
          decorations: matchDecorator.createDeco(view),
          update(update) {
            this.decorations = matchDecorator.updateDeco(
              update,
              this.decorations
            );
          },
        }),
        { decorations: (v) => v.decorations }
      );

      setHighlightExtension(plugin.extension);
    } catch {}
  }, [pattern]);

  return (
    <ReactCodeMirror
      value={testInput}
      height="200px"
      className="leading-10"
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLine: false,
      }}
      extensions={highlightExtension && [highlightExtension]}
      onChange={(e) => setTestInput(e)}
    />
  );
};

export default TestInput;
