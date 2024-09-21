import { ImageResponse } from "next/og";

export const runtime = "edge";

export const alt = "bulloak - Test Generator using Branching Tree Technique";
export const size = {
  width: 1200,
  height: 630,
};

export const contentType = "image/png";

export default async function Image() {
  const satoshi = await fetch(
    new URL("./fonts/Satoshi-Medium.woff", import.meta.url),
  ).then((res) => res.arrayBuffer());
  const commit = await fetch(
    new URL("./fonts/CommitMono-400-Regular.otf", import.meta.url),
  ).then((res) => res.arrayBuffer());

  return new ImageResponse(
    (
      <div
        tw="relative min-h-screen flex flex-col w-full items-center font-normal justify-center px-4 bg-slate-200"
        style={{ fontFamily: "Satoshi Variable" }}
      >
        <div tw="flex flex-col relative text-center">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="currentColor"
            viewBox="0 0 1405.54 1454"
            height="96px"
            style={{ alignSelf: "center" }}
          >
            <path d="m724.07,1454h-81.92v-628.04l-292.02-174.04-186.35,182.81-150.78,2.97-11.77-84.59,63.47.24L0,673.28l14.67-116.55,139.64,171.26,120.33-121.1L39.95,467.3l34.05-78.27,100.48,59.99-22.84-182.77,74.83-60.93,37.63,296.12,377.95,225.4-.54-267.92-272.75-110.43-114.05-187.4,69.8-50.88,80.59,132.1,65.45-203.11,82.82,17.42-78.12,241.94,167.03,65.76.03-30.96-119.92-79.28,28.84-86.86,91.05,65.21.08-232.44h81.69l.15,161.24L899.86,28.01l96.5,36.04-272.28,206.6-.21,436.6,229.98-143.04-54.83-275.39.96-1.33,143.69-199.47,73.72,47.61-101.38,141.93,213.71-40,55.89,73.02-279.17,50.3,29.73,152.53,271.68-168.74,38.87,75.5-142.1,88.34,192.73,83.91,8.21,84.75-288-114.76-125.74,76.34,175.99,142.21,227.43-83.86,6.45,84.83-245.65,94.83-1.69-1.37-234.77-189.96-195.49,120.76v647.79Z" />
          </svg>
          <h1
            tw="mt-6 items-center justify-center text-4xl font-bold"
            style={{ fontFamily: "CommitMono" }}
          >
            bulloak
          </h1>
          <p tw="text-xl max-w-sm mt-4">
            A smart contract test generator based on the Branching Tree
            Technique
          </p>
        </div>
      </div>
    ),
    {
      ...size,
      fonts: [
        {
          name: "Satoshi Variable",
          data: satoshi,
          style: "normal",
        },
        {
          name: "CommitMono",
          data: commit,
          style: "normal",
        },
      ],
    },
  );
}
