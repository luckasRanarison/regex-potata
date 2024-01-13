import { useEffect, useRef, useState } from "react";
import { Viz, instance } from "@viz-js/viz";
import Navbar from "./components/Navbar";
import ExpressionsPopup from "./components/ExpressionsPopup";
import { RiQuestionFill } from "react-icons/ri";
import { RegexEngine, RegexCapture } from "regex-potata";
import { graphFromRegex } from "./utils/viz";
import TestInput from "./components/TestInput";
import Footer from "./components/Footer";
import RegexInput from "./components/RegexInput";
import ToolTip from "./components/ToolTip";
import NfaVisualizer from "./components/NfaVisualizer";
import Loader from "./components/Loader";

const App = () => {
  const [isLoading, setIsLoading] = useState(true);
  const [regexInput, setRegexInput] = useState("");
  const [testInput, setTestInput] = useState("");
  const [regexInstance, setRegexInstance] = useState<RegexEngine>();
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const [svg, setSvg] = useState<SVGSVGElement>();
  const [captures, setCaptures] = useState<RegexCapture[]>([]);
  const vizInstance = useRef<Viz>();

  useEffect(() => {
    (async () => {
      const i = await instance();
      const engine = new RegexEngine("");
      vizInstance.current = i;
      setRegexInstance(engine);
      setTimeout(() => setIsLoading(false), 250);
    })();
  }, []);

  useEffect(() => {
    try {
      setRegexInstance(new RegexEngine(regexInput));
    } catch (error) {
      setRegexInstance(undefined);
    }
  }, [regexInput]);

  useEffect(() => {
    if (regexInstance) {
      const dot = graphFromRegex(regexInstance);
      const elem = vizInstance.current?.renderSVGElement(dot);

      if (elem) {
        setSvg(elem);
      }
    }
  }, [regexInstance]);

  useEffect(() => {
    if (regexInstance) {
      setCaptures(regexInstance.capturesAll(testInput));
    }
  }, [testInput, regexInstance]);

  if (isLoading) {
    return <Loader />;
  }

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
              <ToolTip label="Show help" onClick={() => setIsPopupOpen(true)}>
                <RiQuestionFill />
              </ToolTip>
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
              captures={captures}
              onInput={(v) => setTestInput(v)}
            />
          </div>
          <div className="space-y-10">
            <div className="font-semibold">NFA Visualizer</div>
            <NfaVisualizer svg={svg} />
          </div>
        </div>
      </div>
      <Footer />
      <ExpressionsPopup
        open={isPopupOpen}
        onClose={() => setIsPopupOpen(false)}
      />

      {/* Transition mask */}
      <div
        className="fixed w-screen h-screen z-50 opacity-0
        animate-fade pointer-events-none bg-slate-900"
      ></div>
    </div>
  );
};

export default App;
