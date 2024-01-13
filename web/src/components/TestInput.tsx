import ReactCodeMirror, { Extension } from "@uiw/react-codemirror";
import { useEffect, useState } from "react";
import { RegexCapture, RegexMatch } from "regex-potata";
import { getMatchHighlight, groupHoverTooltip } from "../utils/extensions";

type InputProps = {
  input: string;
  matches: RegexMatch[];
  captures: RegexCapture[];
  onInput: (value: string) => void;
};

const TestInput = ({ input, matches, captures, onInput }: InputProps) => {
  const [highlightExtension, setHighlightExtension] = useState<Extension>();
  const [hoverExtension, setHoverExtension] = useState<Extension>();

  useEffect(() => {
    if (matches.length) {
      setHighlightExtension(getMatchHighlight(matches, captures));
    } else {
      setHighlightExtension(undefined);
    }
  }, [matches]);

  useEffect(() => {
    if (captures.length) {
      setHoverExtension(groupHoverTooltip(captures));
    } else {
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
