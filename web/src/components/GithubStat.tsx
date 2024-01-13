import { IconType } from "react-icons";

type StatProp = {
  icon: IconType;
  path: string;
  number: number;
};

const GithubStat = ({ icon: Icon, path, number }: StatProp) => (
  <a
    href={`https://github.com/luckasRanarison/regex-potata/${path}`}
    className="flex items-center space-x-2 cursor-pointer"
  >
    <Icon className="text-cyan-300" />
    <span className="font-semibold">{number}</span>
  </a>
);

export default GithubStat;
