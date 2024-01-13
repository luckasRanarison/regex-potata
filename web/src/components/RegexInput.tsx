import { copy } from "clipboard";
import { RiCheckFill, RiCloseCircleFill, RiFileCopyLine } from "react-icons/ri";
import ToolTip from "./ToolTip";
import { useState } from "react";

type InputProps = {
  value: string;
  error: boolean;
  onInput: (value: string) => void;
};

const RegexInput = ({ value, error, onInput }: InputProps) => {
  const [hasCopied, setHasCopied] = useState(false);

  const handleCopy = () => {
    setHasCopied(true);
    setTimeout(() => setHasCopied(false), 1500);
    copy(value);
  };

  return (
    <>
      <div className="relative flex items-center">
        <input
          value={value}
          placeholder="Insert a regular expression..."
          onChange={(e) => onInput(e.target.value)}
          className={`py-3 px-5 w-full
          rounded-md border-[1px] border-slate-800
          bg-transparent focus:outline-none focus:border-cyan-300
          ${error && "!border-red-400"}`}
        />
        <div className="absolute right-4">
          <ToolTip label="Copy to clipboard" onClick={handleCopy}>
            {hasCopied ? <RiCheckFill /> : <RiFileCopyLine />}
          </ToolTip>
        </div>
      </div>
      {error && (
        <div
          className="flex items-center space-x-3
          font-semibold text-red-400"
        >
          <RiCloseCircleFill />
          <span>Invalid Regular expression</span>
        </div>
      )}
    </>
  );
};

export default RegexInput;
