#!/usr/bin/env python3
"""Check the project's local trust-label convention (not Rust semantics).

Scope: link processing, its selected link-element caller, and the transitive
MIME specification tree. Dependency sources and generated macro expansions are
part of the separately documented toolchain/library trusted base.

The checker deliberately recognizes source declarations textually. It cannot
prove label truth, executable equivalence, or WHATWG correspondence. --inventory
prints every label grouped by kind for the human review required by the protocol.
"""

import argparse
from collections import Counter, defaultdict
from pathlib import Path
import re
import sys


LABEL = re.compile(r"^\s*//\s+(TRUSTED\((external-lib|scope|verus-gap)\)|REWRITE\(verus-gap\)):\s+(.+)$")
BOUNDARY = re.compile(
    r"verifier::external(?:_body|_type_specification|_trait_specification|_trait_extension)?\b"
    r"|\bassume_specification\b|\bbroadcast\s+axiom\b|\buninterp\s+spec\s+fn\b"
)


def label_before(lines, index):
    """Accept an adjacent comment/attribute block, never cross a code item."""
    for previous in range(index - 1, -1, -1):
        text = lines[previous].strip()
        match = LABEL.match(lines[previous])
        if match:
            return previous, match
        if not text or text.startswith("//") or text.startswith("#["):
            continue
        break
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", action="store_true")
    args = parser.parse_args()
    specs = Path(__file__).resolve().parent
    root = specs.parents[5]
    if not (root / "Cargo.toml").is_file() or not (root / ".git").exists():
        parser.error("expected this script inside the Servo checkout")
    mime_specs = root / "components/shared/net/mime_classifier_specs"
    paths = sorted(set(specs.rglob("*.rs")) | set(mime_specs.rglob("*.rs")) | {
        specs.parent / "processingoptions.rs",
        specs.parent / "htmllinkelement.rs",
        root / "components/shared/net/mime_classifier.rs",
    })
    labels = defaultdict(list)
    checked_items = set()
    errors = []
    rewrites = 0
    for path in paths:
        lines = path.read_text().splitlines()
        relative = path.relative_to(root)
        in_block_comment = False
        for index, line in enumerate(lines):
            text = line.strip()
            match = LABEL.match(line)
            if match:
                kind = match.group(2) or "rewrite"
                labels[kind].append(f"{relative}:{index + 1}: {text}")
                if kind == "rewrite":
                    rewrites += 1
                    if index + 1 == len(lines) or not lines[index + 1].lstrip().startswith("//"):
                        errors.append(f"{relative}:{index + 1}: rewrite lacks immediately following original-code comment")
            if in_block_comment:
                if "*/" in text:
                    in_block_comment = False
                continue
            if text.startswith("/*"):
                in_block_comment = "*/" not in text
                continue
            if text.startswith("//"):
                continue
            if not BOUNDARY.search(text):
                continue
            # Arbitrary mathematical fixture constants assert no native relation.
            if path.name == "tests.rs" and "uninterp spec fn" in text:
                continue
            previous = label_before(lines, index)
            if previous is None:
                errors.append(f"{relative}:{index + 1}: unlabelled boundary: {text}")
            else:
                checked_items.add((relative, previous[0]))
    counts = Counter({kind: len(items) for kind, items in labels.items()})
    if args.inventory:
        for kind in sorted(labels):
            print(f"\n{kind} ({counts[kind]})")
            for entry in labels[kind]:
                print(entry)
    for error in errors:
        print(error, file=sys.stderr)
    print(f"Audited {len(paths)} Rust files; {len(checked_items)} labelled boundary items; {rewrites} rewrites.")
    print("Labels: " + ", ".join(f"{kind}={counts[kind]}" for kind in sorted(counts)))
    print(f"Missing/invalid labels: {len(errors)}")
    return bool(errors)


if __name__ == "__main__":
    sys.exit(main())
