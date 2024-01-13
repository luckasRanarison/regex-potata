export type Stats = {
  stargazers: number;
  watchers: number;
  forks: number;
};

async function getRepoStats(): Promise<Stats> {
  const result = await fetch(
    "https://api.github.com/repos/luckasRanarison/regex-potata"
  );
  const parsed = await result.json();
  const { stargazers_count, watchers_count, forks_count } = parsed;

  return {
    stargazers: stargazers_count,
    watchers: watchers_count,
    forks: forks_count,
  };
}

export { getRepoStats };
