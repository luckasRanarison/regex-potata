import ReactCodeMirror, {
  Decoration,
  Extension,
  RangeSetBuilder,
  ViewPlugin,
} from "@uiw/react-codemirror";
import { useEffect, useState } from "react";
import { RegexMatch } from "regex-potata";

type InputProps = {
  input: string;
  matches: RegexMatch[];
  onInput: (value: string) => void;
};

const decoration = Decoration.mark({
  class: "highlight-chunk",
  inclusiveStart: true,
  inclusiveEnd: false,
});

const TestInput = ({ input, matches, onInput }: InputProps) => {
  const [highlightExtension, setHighlightExtension] = useState<Extension>();

  useEffect(() => {
    if (!matches.length) {
      return setHighlightExtension(undefined);
    }

    const decorationBuilder = new RangeSetBuilder<Decoration>();

    for (const match of matches) {
      decorationBuilder.add(match.start, match.end, decoration);
    }

    const plugin = ViewPlugin.define(
      () => ({
        decorations: decorationBuilder.finish(),
      }),
      { decorations: (plugin) => plugin.decorations }
    );

    setHighlightExtension(plugin.extension);
  }, [matches]);

  return (
    <ReactCodeMirror
      value={input}
      height="200px"
      className="leading-10"
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLine: false,
      }}
      extensions={highlightExtension && [highlightExtension]}
      onChange={(value) => onInput(value)}
    />
  );
};

export default TestInput;
