type ToolTipProps = {
  label: string;
  children: React.ReactNode;
  onClick: () => void;
};

const ToolTip = ({ label, children, onClick }: ToolTipProps) => (
  <button onClick={onClick} className="relative group flex justify-center">
    {children}
    <div
      className="z-50 group-hover:block hidden
      absolute -bottom-14 w-max py-2 px-4 font-normal
      rounded-md text-sm text-white bg-slate-600 shadow-md"
    >
      {label}
    </div>
  </button>
);

export default ToolTip;
