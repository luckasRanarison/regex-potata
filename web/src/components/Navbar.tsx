import { RiGithubFill } from "react-icons/ri";

const Navbar = () => (
  <div
    className="z-40 sticky top-0 py-4 px-6 2xl:px-10 w-full 
    flex justify-between items-center
    border-b-[1px] border-slate-800 bg-slate-900"
  >
    <div className="font-semibold text-2xl">
      <span>Regex </span>
      <span className="text-cyan-300">Potata</span>
    </div>
    <a
      href="https://github.com/luckasRanarison/regex-potata"
      className="flex space-x-2 hover:text-cyan-300"
      target="_blank"
    >
      <span className="font-semibold">Source</span>
      <RiGithubFill size={24} />
    </a>
  </div>
);

export default Navbar;
