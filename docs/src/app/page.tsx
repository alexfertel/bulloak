import Image from "next/image";
import AsciiTreeAnimation from "../components/AsciiTreeAnimation";
import Link from "next/link";

const HeroSection = () => {
  return (
    <div className="relative h-screen flex flex-col items-center justify-center bg-[#f5f5dc] text-black overflow-hidden">
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

const AboutSection = () => {
  return (
    <div className="py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">[ > ] ABOUT BULLOAK</h2>
        <p className="text-lg mb-4">
          BULLOAK IS A SOLIDITY TEST GENERATOR THAT IMPLEMENTS
          THE BRANCHING TREE TECHNIQUE (BTT) FOR COMPREHENSIVE
          SMART CONTRACT TESTING.
        </p>
      </div>
    </div>
  );
};

const FeaturesSection = () => {
  return (
    <div className="py-16 bg-[#f5f5dc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">[ > ] FEATURES</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <FeatureCard
            title="BRANCHING TREE TECHNIQUE"
            description="GENERATE COMPREHENSIVE TEST CASES USING BTT"
          />
          <FeatureCard
            title="SOLIDITY FOCUS"
            description="TAILORED FOR ETHEREUM SMART CONTRACT TESTING"
          />
          <FeatureCard
            title="EASY INTEGRATION"
            description="SEAMLESSLY INTEGRATE WITH EXISTING PROJECTS"
          />
          <FeatureCard
            title="CUSTOMIZABLE"
            description="ADAPT TO YOUR SPECIFIC TESTING NEEDS"
          />
        </div>
      </div>
    </div>
  );
};

const FeatureCard = ({ title, description }) => {
  return (
    <div className="border border-black p-6">
      <h3 className="text-xl font-bold mb-4">{title}</h3>
      <p>{description}</p>
    </div>
  );
};

const UsageSection = () => {
  return (
    <div className="py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">[ > ] USAGE</h2>
        <p className="text-lg mb-4">
          TO USE BULLOAK, FOLLOW THESE STEPS:
        </p>
        <ol className="list-decimal list-inside text-lg mb-8">
          <li>INSTALL BULLOAK VIA NPM OR YARN</li>
          <li>IMPORT BULLOAK INTO YOUR TEST SUITE</li>
          <li>DEFINE YOUR CONTRACT'S FUNCTIONS AND STATES</li>
          <li>GENERATE TEST CASES USING BTT</li>
          <li>RUN THE GENERATED TESTS</li>
        </ol>
        <Link href="/docs" className="text-lg font-bold underline">
          READ THE FULL DOCUMENTATION
        </Link>
      </div>
    </div>
  );
};

const ResourcesSection = () => {
  return (
    <div className="py-16 bg-[#f5f5dc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">[ > ] RESOURCES</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <ResourceLink
            href="https://github.com/alexfertel/bulloak"
            text="GITHUB REPOSITORY"
          />
          <ResourceLink
            href="/docs/btt-explanation"
            text="BTT EXPLANATION"
          />
          <ResourceLink
            href="/docs/getting-started"
            text="GETTING STARTED GUIDE"
          />
          <ResourceLink
            href="/docs/api-reference"
            text="API REFERENCE"
          />
        </div>
      </div>
    </div>
  );
};

const ResourceLink = ({ href, text }) => {
  return (
    <Link href={href} className="block border border-black p-6 hover:bg-black hover:text-[#f5f5dc] transition-colors">
      <span className="text-xl font-bold">{text}</span>
    </Link>
  );
};

export default function Home() {
  return (
    <main className="min-h-screen bg-[#f5f5dc] text-black">
      <HeroSection />
      <AboutSection />
      <FeaturesSection />
      <UsageSection />
      <ResourcesSection />
    </main>
  );
}
