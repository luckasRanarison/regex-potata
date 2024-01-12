import ReactCodeMirror, { Extension } from "@uiw/react-codemirror";
import { useEffect, useState } from "react";
import { RegexMatch } from "regex-potata";
import { getMatchHighlight } from "../utils/extensions";

type InputProps = {
  input: string;
  matches: RegexMatch[];
  onInput: (value: string) => void;
};

const TestInput = ({ input, matches, onInput }: InputProps) => {
  const [highlightExtension, setHighlightExtension] = useState<Extension>();

  useEffect(() => {
    if (!matches.length) {
      return setHighlightExtension(undefined);
    }

    const extension = getMatchHighlight(matches);

    setHighlightExtension(extension);
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
