import { useEffect, useState } from "react";
import { Stats, getRepoStats } from "../utils/github";
import GithubStat from "./GithubStat";
import {
  RiEyeFill,
  RiBugFill,
  RiGitBranchFill,
  RiStarFill,
} from "react-icons/ri";

const Footer = () => {
  const [repoStats, setRepoStats] = useState<Stats>();

  useEffect(() => {
    getRepoStats()
      .then((res) => setRepoStats(res))
      .catch((error) => console.log(error));
  }, []);

  return (
    <div className="flex flex-col pt-6 pb-12 px-4 space-y-6 text-center">
      <div className="flex space-x-1">
        <div className="flex items-center space-x-1">
          <span>Found an issue</span> <RiBugFill className="text-cyan-300" />{" "}
          <span>? Report it </span>
        </div>
        <div>
          <a
            href="https://github.com/luckasRanarison/regex-potata/issues/new"
            className="text-cyan-300"
          >
            here
          </a>
          <span>.</span>
        </div>
      </div>
      {repoStats && (
        <div className="w-full flex justify-center space-x-8">
          <GithubStat
            path="stargazers"
            icon={RiStarFill}
            number={repoStats.stargazers}
          />
          <GithubStat
            path="forks"
            icon={RiGitBranchFill}
            number={repoStats.forks}
          />
          <GithubStat
            path="watchers"
            icon={RiEyeFill}
            number={repoStats.watchers}
          />
        </div>
      )}
      <div>© Licensed under the MIT License.</div>
    </div>
  );
};

export default Footer;
