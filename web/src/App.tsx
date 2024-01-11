import { useEffect, useRef, useState } from "react";
import { Viz, instance } from "@viz-js/viz";
import Navbar from "./components/Navbar";
import ExpressionsPopup from "./components/ExpressionsPopup";
import { RiCloseCircleFill, RiQuestionFill } from "react-icons/ri";
import { RegexEngine } from "regex-potata";
import { dotFromRegex } from "./utils/viz";
import TestInput from "./components/TestInput";

const App = () => {
  const [regexInput, setRegexInput] = useState("");
  const [regexInstance, setRegexInstance] = useState<RegexEngine>();
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const [svg, setSvg] = useState<SVGSVGElement>();
  const vizInstance = useRef<Viz>();

  useEffect(() => {
    instance().then((i) => (vizInstance.current = i));
  }, []);

  useEffect(() => {
    try {
      setRegexInstance(new RegexEngine(regexInput));
    } catch (_) {
      setRegexInstance(undefined);
    }
  }, [regexInput]);

  useEffect(() => {
    if (regexInstance) {
      const dot = dotFromRegex(regexInstance);
      const elem = vizInstance.current?.renderSVGElement(dot);

      if (elem) {
        setSvg(elem);
      }
    }
  }, [regexInstance]);

  return (
    <div
      className="min-h-screen min-w-screen 
      flex flex-col items-center
      text-white bg-slate-900"
    >
      <Navbar />
      <div className="py-5 px-3 w-full flex justify-center">
        <div
          className="py-4 px-4 md:px-8 md:py-6 w-full max-w-2xl space-y-8
          flex flex-col justify-center
          rounded-md"
        >
          <div className="space-y-4">
            <div className="space-x-3 flex items-center font-semibold">
              <div>Regular expression</div>
              <button onClick={() => setIsPopupOpen(true)}>
                <RiQuestionFill />
              </button>
            </div>
            <input
              value={regexInput}
              placeholder="Insert a regular expression..."
              onChange={(e) => setRegexInput(e.target.value)}
              className={`py-3 px-5 w-full
              rounded-md border-[1px] border-slate-800
              bg-transparent focus:outline-none focus:border-cyan-300
              ${!regexInstance && "!border-red-400"}`}
            />
            {!regexInstance && (
              <div className="flex items-center space-x-3 font-semibold text-red-400">
                <RiCloseCircleFill />
                <span>Invalid Regular expression</span>
              </div>
            )}
          </div>
          <div className="space-y-4">
            <div className="font-semibold">Test input</div>
            <TestInput pattern={regexInput} />
          </div>
          <div className="space-y-10">
            <div className="font-semibold">NFA Visualizer</div>
            <div className="w-full overflow-scroll">
              <svg
                height={svg?.height.baseVal.value}
                width={svg?.width.baseVal.value}
                dangerouslySetInnerHTML={{ __html: svg?.innerHTML ?? "" }}
              ></svg>
            </div>
          </div>
        </div>
      </div>
      <ExpressionsPopup
        open={isPopupOpen}
        onClose={() => setIsPopupOpen(false)}
      />
    </div>
  );
};

export default App;
