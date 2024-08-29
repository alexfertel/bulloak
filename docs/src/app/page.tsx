import Image from "next/image";
import Link from "next/link";
import AsciiTreeAnimation from "../components/AsciiTreeAnimation";

export default function Home() {
  return (
    <main className="min-h-screen bg-[#f5f5dc] text-black">
      <HeroSection />
      <BTTExplanationSection />
      <BulloakFeaturesSection />
      <UsageSection />
      <ResourcesSection />
      <Footer />
    </main>
  );
}

const HeroSection = () => {
  return (
    <div className="relative h-screen flex flex-col items-center justify-center bg-[#f5f5dc] text-black overflow-hidden">
      {/* <Image
        src="/bulloak-logo.png"
        alt="Bulloak Logo"
        width={300}
        height={300}
        className="mb-8"
      /> */}
      <h1 className="text-6xl font-bold text-center z-10 mb-4">
        bulloak
      </h1>
      <p className="text-xl text-center z-10 mb-8">
        A Solidity test generator based on the Branching Tree Technique
      </p>
      <div className="flex space-x-4">
        <Link href="/docs" className="bg-black text-[#f5f5dc] px-6 py-2 rounded-md hover:bg-opacity-80">
          Get Started
        </Link>
        <Link href="https://github.com/alexfertel/bulloak" className="border border-black px-6 py-2 rounded-md hover:bg-black hover:text-[#f5f5dc]">
          GitHub
        </Link>
      </div>
      <AsciiTreeAnimation />
    </div>
  );
};

const BTTExplanationSection = () => {
  return (
    <div className="py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">What is the Branching Tree Technique?</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div>
            <p className="text-lg mb-4">
              The Branching Tree Technique (BTT) is a structured approach to test case design that ensures comprehensive coverage of all possible scenarios and edge cases in your smart contract tests.
            </p>
            <p className="text-lg mb-4">
              BTT organizes test cases in a tree-like structure, where:
            </p>
            <ul className="text-lg mb-4">
              <li><span className="mr-2 font-mono">├─</span>Branches represent different conditions or states</li>
              <li><span className="mr-2 font-mono">└─</span>Leaves represent specific test cases or assertions</li>
            </ul>
          </div>
          <div className="border border-black p-6">
            <h3 className="text-xl font-bold mb-4">Example BTT Structure</h3>
            <pre className="text-sm font-mono">
              {`HashPairTest
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of keccak256(a,b).
└── When first arg is bigger than second arg
    └── It should match the result of keccak256(b,a).`}
            </pre>
          </div>
        </div>
        <div className="mt-8">
          <p className="text-lg mb-4">
            This technique helps developers:
          </p>
          <ul className="text-lg mb-4">
            <li><span className="mr-2 font-mono">├─</span>Visualize all possible test scenarios</li>
            <li><span className="mr-2 font-mono">├─</span>Ensure complete test coverage</li>
            <li><span className="mr-2 font-mono">├─</span>Easily identify missing test cases</li>
            <li><span className="mr-2 font-mono">└─</span>Maintain a clear and organized test structure</li>
          </ul>
          <p className="text-lg">
            Bulloak leverages BTT to automatically generate comprehensive Solidity test suites, saving time and improving the quality of smart contract testing.
          </p>
        </div>
      </div>
    </div>
  );
};

const BulloakFeaturesSection = () => {
  return (
    <div className="py-16 bg-[#f5f5dc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">What is bulloak?</h2>
        <p className="text-lg">
          bulloak is a powerful tool that brings the Branching Tree Technique to life for developers. It automates the process of creating comprehensive test suites based on BTT specifications.
          {" "}<Link href="https://github.com/alexfertel/bulloak" target="_blank" className="inline-flex text-lg font-bold underline hover:text-gray-500">
            Read the full README on GitHub.
          </Link>
        </p>
        <pre className="text-lg font-mono">
          <code>{`
Bulloak
├── Scaffold Command
│   ├─── Automatically generates Solidity test files from .tree specifications
│   │   ├── Creates modifiers for conditions
│   │   └── Generates test functions for actions
│   ├─── Reports syntax errors in your specification
│   └─── Provides a full AST for easy extension
├── Check Command
│   ├── Ensures code matches its specification
│   ├── Reports missing tests
│   └── Identifies structural mismatches
├── Multiple Tree Support
│   └── Define multiple test trees in a single file
├── Flexible Syntax
│   ├── Supports 'when' and 'given' for conditions
│   └── Case-insensitive keywords
└── Automatic Fixes
    ├── Adds missing functions
    └── Corrects incorrect ordering
          `}</code>
        </pre>
        <div className="border border-black p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="font-mono">
              <h5 className="text-lg font-semibold mb-2">hash_pair.tree</h5>
              <pre className="border border-black text-gray-900 px-4 overflow-x-auto">
                <code>{`
HashPairTest
├── It should never revert.
├── When first arg is smaller than second arg
│   └── It should match the result of
│       keccak256(abi.encodePacked(a,b)).
└── When first arg is bigger than second arg
    └── It should match the result of
        keccak256(abi.encodePacked(b,a)).
                `}</code>
              </pre>
            </div>
            <div className="font-mono">
              <h5 className="text-lg font-semibold mb-2">CLI</h5>
              <pre className="border border-black text-gray-900 px-4 overflow-x-auto">
                <code>{`
# Generate Solidity tests.
$ bulloak scaffold hash_pair.tree

# Check if tests match the specification.
$ bulloak check hash_pair.tree

# Automatically fix issues.
$ bulloak check --fix hash_pair.tree
                `}</code>
              </pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

const UsageSection = () => {
  return (
    <div className="py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">Usage</h2>
        <p className="text-lg mb-4">
          To use Bulloak, follow these steps:
        </p>
        <ol className="list-decimal list-inside text-lg mb-8">
          <li>Install Bulloak via Cargo</li>
          <li>Create a .tree file with your test specification</li>
          <li>Use 'bulloak scaffold' to generate Solidity test files</li>
          <li>Use 'bulloak check' to ensure your code matches the spec</li>
          <li>Run the generated tests with your preferred Solidity testing framework</li>
        </ol>
        <Link href="https://github.com/alexfertel/bulloak" target="_blank" className="inline-block text-lg font-bold underline hover:text-gray-500">
          Read the full README on GitHub.
        </Link>
      </div>
    </div>
  );
};

const ResourcesSection = () => {
  return (
    <div className="py-16 bg-[#f5f5dc] text-black">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold mb-8">Resources</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <ResourceLink
            href="https://github.com/alexfertel/bulloak"
            text="GitHub Repository"
          />
          <ResourceLink
            href="/docs/btt-explanation"
            text="BTT Explanation"
          />
          <ResourceLink
            href="/docs/getting-started"
            text="Getting Started Guide"
          />
          <ResourceLink
            href="/docs/api-reference"
            text="API Reference"
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

const Footer: React.FC = () => {
  return (
    <footer className="bg-[#f5f5dc] text-black py-8 border-t border-black">
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 md:grid-cols-5 gap-8">
          <div className="col-span-2">
            <h3 className="text-xl font-bold mb-4">Tools & Infrastructure/</h3>
            <ul className="space-y-2">
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Hiro Platform</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Clarinet</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Clarity VSCode Extension</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Stacks.js</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Stacks Blockchain API</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Token Metadata API</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Ordinals API</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Chainhook</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Ordhook</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Stacks Explorer</Link></li>
              <li><span className="mr-2">└─</span><Link href="/" className="hover:underline">Ordinals Explorer</Link></li>
            </ul>
          </div>
          <div>
            <h3 className="text-xl font-bold mb-4">Build/</h3>
            <ul className="space-y-2">
              <li><span className="mr-2">├─</span><Link href="/docs" className="hover:underline">Documentation</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Example apps</Link></li>
              <li><span className="mr-2">└─</span><Link href="/" className="hover:underline">Tutorials</Link></li>
            </ul>
          </div>
          <div>
            <h3 className="text-xl font-bold mb-4">Resources/</h3>
            <ul className="space-y-2">
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Blog</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Clarity Playground</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Videos</Link></li>
              <li><span className="mr-2">└─</span><Link href="/" className="hover:underline">Newsletter</Link></li>
            </ul>
          </div>
          <div>
            <h3 className="text-xl font-bold mb-4">Company/</h3>
            <ul className="space-y-2">
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Careers <span className="text-red-500">we're hiring</span></Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">About us</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Press</Link></li>
              <li><span className="mr-2">└─</span><Link href="/" className="hover:underline">Bounty program</Link></li>
            </ul>
            <h3 className="text-xl font-bold mt-6 mb-4">Social/</h3>
            <ul className="space-y-2">
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">X</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">Join Discord</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">GitHub</Link></li>
              <li><span className="mr-2">├─</span><Link href="/" className="hover:underline">YouTube</Link></li>
              <li><span className="mr-2">└─</span><Link href="/" className="hover:underline">LinkedIn</Link></li>
            </ul>
          </div>
        </div>
        <div className="mt-12">
          <h3 className="text-xl font-bold mb-4">STAY UP TO DATE WITH PRODUCT UPDATES, LEARNING RESOURCES, AND MORE.</h3>
          <div className="flex">
            <input type="email" placeholder="YOUR EMAIL" className="flex-grow p-2 border border-black mr-2" />
            <button className="bg-black text-[#f5f5dc] p-2 hover:bg-opacity-80">[ SUBSCRIBE ]</button>
          </div>
        </div>
        <div className="mt-12 text-sm">
          <Link href="/" className="hover:underline mr-4">Patent Pledge</Link>
          <Link href="/" className="hover:underline mr-4">Terms of Use</Link>
          <Link href="/" className="hover:underline mr-4">Privacy</Link>
          <span className="float-right">
            <span className="mr-2">Status</span>
            <span className="text-green-500">● All Systems Operational</span>
            <span className="ml-4">© 2024 Hiro Systems PBC</span>
          </span>
        </div>
      </div>
    </footer>
  );
};
