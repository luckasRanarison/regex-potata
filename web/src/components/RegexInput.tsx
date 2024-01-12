import { RiCloseCircleFill } from "react-icons/ri";

type InputProps = {
  value: string;
  error: boolean;
  onInput: (value: string) => void;
};

const RegexInput = ({ value, error, onInput }: InputProps) => (
  <>
    <input
      value={value}
      placeholder="Insert a regular expression..."
      onChange={(e) => onInput(e.target.value)}
      className={`py-3 px-5 w-full
      rounded-md border-[1px] border-slate-800
      bg-transparent focus:outline-none focus:border-cyan-300
      ${error && "!border-red-400"}`}
    />
    {error && (
      <div className="flex items-center space-x-3 font-semibold text-red-400">
        <RiCloseCircleFill />
        <span>Invalid Regular expression</span>
      </div>
    )}
  </>
);

export default RegexInput;
