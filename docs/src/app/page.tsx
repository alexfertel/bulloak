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
      <Footer />
    </main>
  );
}


const HeroSection = () => {
  return (
    <div className="relative h-screen flex items-center justify-center">
      <div className="absolute inset-0 filter blur-[4px]">
        <AsciiTreeAnimation />
      </div>
      <div className="relative z-10 text-center">
        <h1 className="text-6xl font-bold mb-4">Bulloak</h1>
        <p className="text-xl mb-8">
          A Solidity test generator based on the Branching Tree Technique
        </p>
        <Link href="https://github.com/alexfertel/bulloak" target="_blank" className="bg-black text-[#f5f5dc] px-6 py-2 rounded-md hover:bg-opacity-80">
          Get Started
        </Link>
      </div>
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
              <li><span className="mr-2 font-mono">├──</span>Branches represent different conditions or states</li>
              <li><span className="mr-2 font-mono">└──</span>Leaves represent specific test cases or assertions</li>
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
            <li><span className="mr-2 font-mono">├──</span>Visualize all possible test scenarios</li>
            <li><span className="mr-2 font-mono">├──</span>Ensure complete test coverage</li>
            <li><span className="mr-2 font-mono">├──</span>Easily identify missing test cases</li>
            <li><span className="mr-2 font-mono">└──</span>Maintain a clear and organized test structure</li>
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
        <div className="space-y-6">
          <div>
            <h3 className="text-xl font-semibold mb-2">1. Install Bulloak</h3>
            <pre className="border border-black text-gray-900 px-4 overflow-x-auto py-2">
              <code>$ cargo install bulloak</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">2. Create a .tree file (e.g., foo.tree)</h3>
            <pre className="border border-black text-gray-900 px-4 overflow-x-auto py-2">
              <code>{`FooTest
└── When stuff is called
    └── When a condition is met
        └── It should revert.
            └── Because we shouldn't allow it.`}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">3. Generate Solidity test files</h3>
            <pre className="border border-black text-gray-900 px-4 overflow-x-auto py-2">
              <code>{`$ bulloak scaffold foo.tree
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract FooTest {
    modifier whenStuffIsCalled() {
        _;
    }

    function test_RevertWhen_AConditionIsMet() external whenStuffIsCalled {
        // It should revert.
        //     Because we shouldn't allow it.
    }
}
`}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">4. Check if code matches the spec</h3>
            <pre className="border border-black text-gray-900 px-4 overflow-x-auto py-2">
              <code>{`$ bulloak check foo.tree
warn: function "test_WhenFirstArgIsBiggerThanSecondArg" is missing in .sol
     + fix: run \`bulloak check --fix foo.tree\`
   --> foo.tree:5

warn: 1 check failed (run \`bulloak check --fix foo.tree\` to apply 1 fix)`}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">5. Automatically fix issues (if any)</h3>
            <pre className="border border-black text-gray-900 px-4 overflow-x-auto py-2">
              <code>{`$ bulloak check --fix foo.tree
...
success: 1 issue fixed.`}</code>
            </pre>
          </div>
        </div>
        <p className="mt-8 text-lg">
          For more detailed usage instructions and options, refer to the{" "}
          <Link href="https://github.com/alexfertel/bulloak" target="_blank" className="underline hover:text-gray-500">
            full documentation on GitHub
          </Link>.
        </p>
      </div>
    </div>
  );
};

const Footer = () => {
  return (
    <footer className="bg-[#f5f5dc] text-black pt-14 pb-8">
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="col-span-1">
            <h3 className="text-xl font-bold mb-4">Resources/</h3>
            <ul>
              <li><span className="mr-2">├─</span><Link href="https://www.youtube.com/watch?v=V6KBy8QQnCo" target="_blank" className="hover:underline hover:text-gray-500">Presentation by Paul R. Berg at EthCC[6]</Link></li>
              <li><span className="mr-2">├─</span><Link href="https://www.youtube.com/watch?v=V6KBy8QQnCo" target="_blank" className="hover:underline hover:text-gray-500">Presentation by Paul R. Berg at Devconnect</Link></li>
              <li><span className="mr-2">├─</span><Link href="https://github.com/PaulRBerg/btt-examples" target="_blank" className="hover:underline hover:text-gray-500">BTT examples</Link></li>
              <li><span className="mr-2">├─</span><Link href="https://marketplace.visualstudio.com/items?itemName=aprilandjan.ascii-tree-generator" target="_blank" className="hover:underline hover:text-gray-500">Ascii Tree Generator for VSCode</Link></li>
              <li><span className="mr-2">└─</span><Link href="https://marketplace.visualstudio.com/items?itemName=PraneshASP.vscode-solidity-inspector" className="hover:underline hover:text-gray-500">Syntax highlighting for tree files for VSCode</Link></li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-xl font-bold mb-4">Related Projects/</h3>
            <ul>
              <li><span className="mr-2">├─</span><Link href="https://github.com/marketplace/actions/bulloak-toolchain" target="_blank" className="hover:underline hover:text-gray-500">Run bulloak as a GitHub Action</Link></li>
              <li><span className="mr-2">└─</span><Link href="https://github.com/ericnordelo/poinciana" target="_blank" className="hover:underline hover:text-gray-500">Bulloak for Cairo</Link></li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-xl font-bold mb-4">Supported By/</h3>
            <ul>
              <li><span className="mr-2">├─</span><Link href="https://www.rumpel.xyz/" target="_blank" className="hover:underline hover:text-gray-500">Rumpel Labs</Link></li>
              <li><span className="mr-2">└─</span><Link href="https://sablier.com/" target="_blank" className="hover:underline hover:text-gray-500">Sablier</Link></li>
            </ul>
          </div>
        </div>
        <div className="mt-12 text-sm flex items-center justify-end">
          Created by <Link href="https://alexfertel.me" target="_blank" className="ml-1 text-blue-700 hover:underline font-mono">alexfertel</Link>.
        </div>
      </div>
    </footer>
  );
};
