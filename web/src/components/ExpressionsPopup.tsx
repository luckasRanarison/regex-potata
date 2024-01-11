import { Dialog } from "@headlessui/react";
import { useRef } from "react";

const Snippet = ({ children }: { children: React.ReactNode }) => (
  <span className="py-1 px-2 rounded-md text-sm text-slate-900 bg-cyan-100">
    {children}
  </span>
);

const expressions = [
  { desc: "Basic regex", pat: ["foo", "(bar)", "foo|bar", "fo."] },
  { desc: "Quantifiers", pat: ["+", "*", "?", "{x}", "{x,y}", "{x,}"] },
  {
    desc: "Character class",
    pat: ["a-z]", "[^x]", "\\d", "\\D", "\\w", "\\W", "\\s", "\\S"],
  },
  { desc: "Capture groups", pat: ["(foo)", "(:?bar)", "(?<name>foo)"] },
];

type PopupProps = {
  open: boolean;
  onClose: () => void;
};

const ExpressionsPopup = ({ open, onClose }: PopupProps) => {
  const popupRef = useRef(null);

  return (
    <Dialog initialFocus={popupRef} open={open} onClose={onClose}>
      <div
        className="fixed inset-0 p-4
        flex w-screen items-center justify-center
        backdrop-blur-sm bg-[#00000080]"
      >
        <Dialog.Panel
          ref={popupRef}
          className="w-full max-w-md relative py-6 px-6 space-y-5
          rounded-md text-white bg-slate-900"
          onClick={onClose}
        >
          <Dialog.Title className="text-xl font-semibold">
            Supported expressions
          </Dialog.Title>
          <ul className="space-y-3 list-inside list-disc">
            {expressions.map(({ desc, pat }, key) => (
              <li key={key} className="space-y-2">
                <span>{desc}: </span>
                <span className="flex flex-wrap gap-3">
                  {pat.map((snip, key) => (
                    <Snippet key={key}>{snip}</Snippet>
                  ))}
                </span>
              </li>
            ))}
          </ul>
        </Dialog.Panel>
      </div>
    </Dialog>
  );
};

export default ExpressionsPopup;
