import Image from "next/image";
import AsciiTreeAnimation from "../components/AsciiTreeAnimation";

const HeroSection = () => {
  return (
    <div className="relative h-screen flex flex-col items-center justify-center bg-black text-white overflow-hidden">
      <h1 className="text-6xl font-bold text-center z-10 mb-4">
        bulloak
      </h1>
      <p className="text-xl text-center z-10 mb-8">
        A Solidity test generator based on the Branching Tree Technique
      </p>
      <AsciiTreeAnimation />
    </div>
  );
};

export default function Home() {
  return (
    <main className="min-h-screen bg-black text-white">
      <HeroSection />
      {/* Add more sections here */}
    </main>
  );
}
