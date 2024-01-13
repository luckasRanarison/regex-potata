const Loader = () => (
  <div
    className="w-screen h-screen bg-slate-900
    flex items-center justify-center"
  >
    <div
      className="absolute right-1/2 bottom-1/2
      transform translate-x-1/2 translate-y-1/2 "
    >
      <div
        className="border-solid border-cyan-300 border-4
        border-t-transparent animate-spin rounded-full h-20 w-20"
      ></div>
    </div>
  </div>
);

export default Loader;
