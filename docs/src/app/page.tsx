import Link from "next/link";
import AsciiTreeAnimation from "../components/TreesAnimation";
import { BulloakIcon, GitHubIcon } from "@/components/icons";

export default function Home() {
  return (
    <main className="min-h-screen bg-slate-100 text-slate-900">
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
    <div className="relative min-h-screen flex items-center justify-center px-4 sm:px-6 lg:px-8 bg-slate-200">
      <div className="absolute inset-0 filter blur-[4px]">
        <AsciiTreeAnimation />
      </div>
      <div className="relative z-10 text-center">
        <BulloakIcon className="w-full h-24" />
        <h1 className="mt-6 inline-flex items-center justify-center text-4xl sm:text-5xl md:text-6xl font-bold font-mono">
          bulloak
        </h1>
        <p className="text-lg sm:text-xl max-w-lg mt-4">
          A smart contract test generator based on the Branching Tree Technique
        </p>
        <Link
          href="https://github.com/alexfertel/bulloak"
          target="_blank"
          className="mt-10 inline-flex items-center justify-center bg-slate-800 text-slate-100 px-4 sm:px-6 py-2 rounded-md hover:bg-slate-700 text-sm sm:text-base"
        >
          <GitHubIcon className="w-4 h-4 mr-2" />
          GitHub
        </Link>
      </div>
    </div>
  );
};

const BTTExplanationSection = () => {
  return (
    <div className="py-12 sm:py-16 bg-slate-50 text-slate-900">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">
          What is the Branching Tree Technique?
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 sm:gap-8">
          <div>
            <p className="text-base sm:text-lg mb-4">
              The Branching Tree Technique (BTT) is a structured approach to
              test case design that ensures comprehensive coverage of all
              possible scenarios and edge cases in your smart contract tests.
            </p>
            <p className="text-base sm:text-lg mb-4">
              BTT organizes test cases in a tree-like structure, where:
            </p>
            <ul className="text-base sm:text-lg mb-4">
              <li>
                <span className="mr-2 font-mono">├──</span>Branches represent
                different conditions or states
              </li>
              <li>
                <span className="mr-2 font-mono">└──</span>Leaves represent
                specific test cases or assertions
              </li>
            </ul>
          </div>
          <div className="border border-slate-300 p-4 sm:p-6">
            <h3 className="text-lg sm:text-xl font-bold mb-4">
              Example BTT Structure
            </h3>
            <pre className="text-xs sm:text-sm font-mono overflow-x-auto">
              {`HashPairTest
├── When first arg is smaller than second arg
│   └── It should match the result of keccak256(a,b).
└── When first arg is bigger than second arg
    └── It should match the result of keccak256(b,a).`}
            </pre>
          </div>
        </div>
        <div className="mt-8">
          <p className="text-lg mb-4">This technique helps developers:</p>
          <ul className="text-lg mb-4">
            <li>
              <span className="mr-2 font-mono">├──</span>Visualize all possible
              test scenarios
            </li>
            <li>
              <span className="mr-2 font-mono">├──</span>Ensure complete test
              coverage
            </li>
            <li>
              <span className="mr-2 font-mono">├──</span>Easily identify missing
              test cases
            </li>
            <li>
              <span className="mr-2 font-mono">└──</span>Maintain a clear and
              organized test structure
            </li>
          </ul>
          <p className="text-lg">
            Bulloak leverages BTT to automatically generate comprehensive test
            suites, saving time and improving the quality of smart contract
            testing.
          </p>
        </div>
      </div>
    </div>
  );
};

const BulloakFeaturesSection = () => {
  return (
    <div className="py-12 sm:py-16 bg-slate-100 text-slate-900">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">
          What is bulloak?
        </h2>
        <p className="text-base sm:text-lg mb-6">
          bulloak is a powerful tool that brings the Branching Tree Technique to
          life for developers. It automates the process of creating
          comprehensive test suites based on BTT specifications.{" "}
          <Link
            href="https://github.com/alexfertel/bulloak"
            target="_blank"
            className="inline-flex text-base sm:text-lg font-bold underline hover:text-slate-600"
          >
            Check out the full README on GitHub.
          </Link>
        </p>
        <pre className="text-sm sm:text-base font-mono overflow-x-auto mb-6 px-4 bg-slate-200 rounded">
          <code>{`
Bulloak
├── Scaffold Command
│   ├─── Generates test files from .tree specifications
│   │   ├── Creates modifiers for conditions
│   │   └── Generates test functions for actions
│   ├─── Reports syntax errors in your specification
│   └─── Provides a full AST for easy extension
├── Check Command
│   ├── Ensures implementation matches its specification
│   ├── Reports missing tests
│   └── Identifies structural mismatches
├── Multiple Tree Support
│   └── Define multiple test trees in a single file
├── Flexible Syntax
│   ├── Supports 'when' and 'given' for conditions and states, respectively
│   └── Case-insensitive keywords
└── Automatic Fixes
    ├── Adds missing functions
    └── Corrects incorrect ordering
          `}</code>
        </pre>
        <div className="border border-slate-300 p-4 sm:p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 sm:gap-8">
            <div className="font-mono">
              <h5 className="text-base sm:text-lg font-semibold mb-2">
                hash_pair.tree
              </h5>
              <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto text-xs sm:text-sm">
                <code>{`
HashPairTest
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
              <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto text-xs sm:text-sm">
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
    <div className="py-12 sm:py-16 bg-slate-50 text-slate-900">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <h2 className="text-2xl sm:text-3xl font-bold mb-6 sm:mb-8">Usage</h2>
        <div className="space-y-6">
          <div>
            <h3 className="text-xl font-semibold mb-2">1. Install Bulloak</h3>
            <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto py-2">
              <code>$ cargo install bulloak</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">
              2. Create a .tree file (e.g., foo.tree)
            </h3>
            <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto py-2">
              <code>{`FooTest
└── When stuff is called
    └── When a condition is met
        └── It should revert.
            └── Because we shouldn't allow it.`}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">
              3. Generate Solidity test files
            </h3>
            <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto py-2">
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
            <h3 className="text-xl font-semibold mb-2">
              4. Check if the code matches the specification
            </h3>
            <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto py-2">
              <code>{`$ bulloak check foo.tree
warn: function "test_WhenFirstArgIsBiggerThanSecondArg" is missing in .sol
     + fix: run \`bulloak check --fix foo.tree\`
   --> foo.tree:5

warn: 1 check failed (run \`bulloak check --fix foo.tree\` to apply 1 fix)`}</code>
            </pre>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">
              5. Automatically fix issues (if any)
            </h3>
            <pre className="border border-slate-300 text-slate-900 px-4 overflow-x-auto py-2">
              <code>{`$ bulloak check --fix foo.tree
...
success: 1 issue fixed.`}</code>
            </pre>
          </div>
        </div>
        <p className="mt-6 sm:mt-8 text-base sm:text-lg">
          For more detailed usage instructions and options, refer to the{" "}
          <Link
            href="https://github.com/alexfertel/bulloak"
            target="_blank"
            className="underline hover:text-slate-600"
          >
            full documentation on GitHub
          </Link>
          .
        </p>
      </div>
    </div>
  );
};

const Footer = () => {
  return (
    <footer className="bg-slate-200 text-slate-900 pt-12 sm:pt-14 pb-6 sm:pb-8">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8">
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Resources/</h3>
            <ul>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://github.com/PaulRBerg/btt-examples"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  BTT examples
                </Link>
              </li>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://youtu.be/V6KBy8QQnCo"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Paul Berg&apos;s presentation at EthCC[6]
                </Link>
              </li>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://youtu.be/0-EmbNVgFA4"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Paul Berg&apos;s presentation at Solidity Summit
                </Link>
              </li>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://github.com/pcaversaccio/createx/tree/ee26a86d9cb9d1fcebef2d0d4be2c1528b1541eb"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Practical Example: CreateX
                </Link>
              </li>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://marketplace.visualstudio.com/items?itemName=aprilandjan.ascii-tree-generator"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Ascii Tree Generator for VSCode
                </Link>
              </li>
              <li>
                <span className="mr-2">└──</span>
                <Link
                  href="https://marketplace.visualstudio.com/items?itemName=PraneshASP.vscode-solidity-inspector"
                  className="hover:underline hover:text-slate-600"
                >
                  Syntax highlighting for tree files for VSCode
                </Link>
              </li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">
              Related Projects/
            </h3>
            <ul>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://github.com/marketplace/actions/bulloak-toolchain"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Run bulloak as a GitHub Action
                </Link>
              </li>
              <li>
                <span className="mr-2">└──</span>
                <Link
                  href="https://github.com/ericnordelo/poinciana"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Bulloak for Cairo
                </Link>
              </li>
            </ul>
          </div>
          <div className="col-span-1">
            <h3 className="text-lg sm:text-xl font-bold mb-4">Supported By/</h3>
            <ul>
              <li>
                <span className="mr-2">├──</span>
                <Link
                  href="https://www.rumpel.xyz/"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Rumpel Labs
                </Link>
              </li>
              <li>
                <span className="mr-2">└──</span>
                <Link
                  href="https://sablier.com/"
                  target="_blank"
                  className="hover:underline hover:text-slate-600"
                >
                  Sablier
                </Link>
              </li>
            </ul>
          </div>
        </div>
        <div className="mt-10 sm:mt-12 text-xs sm:text-sm flex items-center justify-end">
          Created by{" "}
          <Link
            href="https://alexfertel.me"
            target="_blank"
            className="ml-1 text-blue-600 hover:underline font-mono"
          >
            alexfertel
          </Link>
        </div>
      </div>
    </footer>
  );
};
