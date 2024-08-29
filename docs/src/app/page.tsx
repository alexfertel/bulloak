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

interface IconProps {
  className?: string;
}

function GitHubIcon(props: IconProps): JSX.Element {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      fill="currentColor"
      className="w-6 h-6"
      {...props}
    >
      <path
        fillRule="evenodd"
        d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
      ></path>
    </svg>
  );
}

const HeroSection = () => {
  return (
    <div className="relative min-h-screen flex items-center justify-center px-4 sm:px-6 lg:px-8">
      <div className="absolute inset-0 filter blur-[4px]">
        <AsciiTreeAnimation />
      </div>
      <div className="relative z-10 text-center">
        <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold font-mono mb-4">bulloak</h1>
        <p className="text-lg sm:text-xl mb-8">
          A test generator based on the Branching Tree Technique
        </p>
        <Link href="https://github.com/alexfertel/bulloak" target="_blank" className="inline-flex items-center justify-center bg-black text-[#f5f5dc] px-4 sm:px-6 py-2 rounded-md hover:bg-opacity-80 text-sm sm:text-base">
          <GitHubIcon className="w-4 h-4 mr-2" />
          GitHub
        </Link>
      </div>
    </div>
  );
};

const BTTExplanationSection = () => {
  return (
    <div className="py-12 sm:py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">What is the Branching Tree Technique?</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 sm:gap-8">
          <div>
            <p className="text-base sm:text-lg mb-4">
              The Branching Tree Technique (BTT) is a structured approach to test case design that ensures comprehensive coverage of all possible scenarios and edge cases in your smart contract tests.
            </p>
            <p className="text-base sm:text-lg mb-4">
              BTT organizes test cases in a tree-like structure, where:
            </p>
            <ul className="text-base sm:text-lg mb-4">
              <li><span className="mr-2 font-mono">├──</span>Branches represent different conditions or states</li>
              <li><span className="mr-2 font-mono">└──</span>Leaves represent specific test cases or assertions</li>
            </ul>
          </div>
          <div className="border border-black p-4 sm:p-6">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Example BTT Structure</h3>
            <pre className="text-xs sm:text-sm font-mono overflow-x-auto">
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
            Bulloak leverages BTT to automatically generate comprehensive test suites, saving time and improving the quality of smart contract testing.
          </p>
        </div>
      </div>
    </div>
  );
};

const BulloakFeaturesSection = () => {
  return (
    <div className="py-12 sm:py-16 bg-[#f5f5dc] text-black">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">What is bulloak?</h2>
        <p className="text-base sm:text-lg mb-6">
          bulloak is a powerful tool that brings the Branching Tree Technique to life for developers. It automates the process of creating comprehensive test suites based on BTT specifications.
          {" "}<Link href="https://github.com/alexfertel/bulloak" target="_blank" className="inline-flex text-base sm:text-lg font-bold underline hover:text-gray-500">
            Read the full README on GitHub.
          </Link>
        </p>
        <pre className="text-sm sm:text-base font-mono overflow-x-auto mb-6 px-4 bg-[#e6e6cc] rounded">
          <code>{`
Bulloak
├── Scaffold Command
│   ├─── Automatically generates test files from .tree specifications
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
        <div className="border border-black p-4 sm:p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 sm:gap-8">
            <div className="font-mono">
              <h5 className="text-base sm:text-lg font-semibold mb-2">hash_pair.tree</h5>
              <pre className="border border-black text-gray-900 px-4 overflow-x-auto text-xs sm:text-sm">
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
              <h5 className="text-base sm:text-lg font-semibold mb-2">CLI</h5>
              <pre className="border border-black text-gray-900 px-4 overflow-x-auto text-xs sm:text-sm">
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
    <div className="py-12 sm:py-16 bg-[#e6e6cc] text-black">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">Usage</h2>
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
            <h3 className="text-xl font-semibold mb-2">4. Check if the code matches the specification</h3>
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
        <p className="mt-6 sm:mt-8 text-base sm:text-lg">
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
    <footer className="bg-[#f5f5dc] text-black pt-12 sm:pt-14 pb-6 sm:pb-8">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8">
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Resources/</h3>
            <ul>
              <li><span className="mr-2">├──</span><Link href="https://www.youtube.com/watch?v=V6KBy8QQnCo" target="_blank" className="hover:underline hover:text-gray-500">Presentation by Paul R. Berg at EthCC[6]</Link></li>
              <li><span className="mr-2">├──</span><Link href="https://www.youtube.com/watch?v=V6KBy8QQnCo" target="_blank" className="hover:underline hover:text-gray-500">Presentation by Paul R. Berg at Devconnect</Link></li>
              <li><span className="mr-2">├──</span><Link href="https://github.com/PaulRBerg/btt-examples" target="_blank" className="hover:underline hover:text-gray-500">BTT examples</Link></li>
              <li><span className="mr-2">├──</span><Link href="https://marketplace.visualstudio.com/items?itemName=aprilandjan.ascii-tree-generator" target="_blank" className="hover:underline hover:text-gray-500">Ascii Tree Generator for VSCode</Link></li>
              <li><span className="mr-2">└──</span><Link href="https://marketplace.visualstudio.com/items?itemName=PraneshASP.vscode-solidity-inspector" className="hover:underline hover:text-gray-500">Syntax highlighting for tree files for VSCode</Link></li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Related Projects/</h3>
            <ul>
              <li><span className="mr-2">├──</span><Link href="https://github.com/marketplace/actions/bulloak-toolchain" target="_blank" className="hover:underline hover:text-gray-500">Run bulloak as a GitHub Action</Link></li>
              <li><span className="mr-2">└──</span><Link href="https://github.com/ericnordelo/poinciana" target="_blank" className="hover:underline hover:text-gray-500">Bulloak for Cairo</Link></li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Supported By/</h3>
            <ul>
              <li><span className="mr-2">├──</span><Link href="https://www.rumpel.xyz/" target="_blank" className="hover:underline hover:text-gray-500">Rumpel Labs</Link></li>
              <li><span className="mr-2">└──</span><Link href="https://sablier.com/" target="_blank" className="hover:underline hover:text-gray-500">Sablier</Link></li>
            </ul>
          </div>
        </div>
        <div className="mt-10 sm:mt-12 text-xs sm:text-sm flex items-center justify-end">
          Created by <Link href="https://alexfertel.me" target="_blank" className="ml-1 text-blue-700 hover:underline font-mono">alexfertel</Link>.
        </div>
      </div>
    </footer>
  );
};
