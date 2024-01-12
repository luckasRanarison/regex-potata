import { useEffect, useRef, useState } from "react";
import { Viz, instance } from "@viz-js/viz";
import Navbar from "./components/Navbar";
import ExpressionsPopup from "./components/ExpressionsPopup";
import { RiQuestionFill } from "react-icons/ri";
import { OwnedMatch, RegexEngine } from "regex-potata";
import { dotFromRegex } from "./utils/viz";
import TestInput from "./components/TestInput";
import Footer from "./components/Footer";
import RegexInput from "./components/RegexInput";

const App = () => {
  const [regexInput, setRegexInput] = useState("");
  const [testInput, setTestInput] = useState("");
  const [regexInstance, setRegexInstance] = useState<RegexEngine>();
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const [svg, setSvg] = useState<SVGSVGElement>();
  const [matches, setMatches] = useState<OwnedMatch[]>([]);
  const vizInstance = useRef<Viz>();

  useEffect(() => {
    (async () => {
      const i = await instance();
      const engine = new RegexEngine("");
      vizInstance.current = i;
      setRegexInstance(engine);
    })();
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

  useEffect(() => {
    if (regexInstance) {
      setMatches(regexInstance.findAll(testInput));
    }
  }, [testInput, regexInstance]);

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
          flex flex-col justify-center"
        >
          <div className="space-y-4">
            <div className="space-x-3 flex items-center font-semibold">
              <div>Regular expression</div>
              <button onClick={() => setIsPopupOpen(true)}>
                <RiQuestionFill />
              </button>
            </div>
            <RegexInput
              value={regexInput}
              error={!regexInstance}
              onInput={(v) => setRegexInput(v)}
            />
          </div>
          <div className="space-y-4">
            <div className="font-semibold">Test input</div>
            <TestInput
              input={testInput}
              matches={matches}
              onInput={(v) => setTestInput(v)}
            />
          </div>
          <div className="space-y-10">
            <div className="font-semibold">NFA Visualizer</div>
            <div className="w-full overflow-scroll">
              {svg && (
                <svg
                  height={svg?.height.baseVal.value}
                  width={svg?.width.baseVal.value}
                  dangerouslySetInnerHTML={{ __html: svg.innerHTML }}
                ></svg>
              )}
            </div>
          </div>
        </div>
      </div>
      <Footer />
      <ExpressionsPopup
        open={isPopupOpen}
        onClose={() => setIsPopupOpen(false)}
      />
    </div>
  );
};

export default App;
