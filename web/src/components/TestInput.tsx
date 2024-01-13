import ReactCodeMirror, { Extension } from "@uiw/react-codemirror";
import { useEffect, useState } from "react";
import { RegexCapture } from "regex-potata";
import { getMatchHighlight, groupHoverTooltip } from "../utils/extensions";

type InputProps = {
  input: string;
  captures: RegexCapture[];
  onInput: (value: string) => void;
};

const TestInput = ({ input, captures, onInput }: InputProps) => {
  const [highlightExtension, setHighlightExtension] = useState<Extension>();
  const [hoverExtension, setHoverExtension] = useState<Extension>();

  useEffect(() => {
    if (captures.length) {
      setHighlightExtension(getMatchHighlight(captures));
      setHoverExtension(groupHoverTooltip(input, captures));
    } else {
      setHighlightExtension(undefined);
      setHoverExtension(undefined);
    }
  }, [captures]);

  return (
    <ReactCodeMirror
      value={input}
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLine: false,
        bracketMatching: false,
        closeBrackets: false,
      }}
      extensions={[hoverExtension!, highlightExtension!].filter((v) => v)}
      onChange={(value) => onInput(value)}
    />
  );
};

export default TestInput;
