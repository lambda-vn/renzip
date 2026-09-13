#!/usr/bin/env python2
# -*- coding: utf-8 -*-
"""Write tests/fixtures/renpy6-3.0.rpa with the archiver of a real Ren'Py SDK.

Every other fixture is written by make_fixtures.py under Python 3. Ren'Py up
to 7.x ran on Python 2, whose pickles tell byte strings (SHORT_BINSTRING) and
unicode strings (BINUNICODE) apart: the prefix of every entry is the byte
string "", and a name can be either. This script loads the Archive class of
the SDK's own launcher/game/archiver.rpy, as it is, and runs it under the
SDK's own Python 2, so the fixture is what that engine release really writes.

It needs a Ren'Py 6.18 SDK and its interpreter, whose standard library ships
compiled only (hence -O) and without a site module (hence -S). On Windows:

    set PYTHONPATH=<sdk>\\lib\\pythonlib2.7
    <sdk>\\lib\\windows-i686\\python.exe -O -S scripts/make_python2_fixture.py <sdk>

make_fixtures.py writes the expectations for this fixture from MEMBERS
below, and never rewrites the archive itself, so it does not need the SDK.
"""

import os
import shutil
import sys
import tempfile
import textwrap

FIXTURE = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "tests", "fixtures", "renpy6-3.0.rpa"
)

# One unicode name, one byte-string name, and one unicode name outside ASCII:
# the three ways a Python 2 Ren'Py can hand a name to its archiver.
MEMBERS = [
    (u"script.rpy", b'label start:\n    "Hello."\n    return\n'),
    ("images/bg.png", bytes(bytearray(range(256)))),
    (u"café.txt", b"accent\n"),
]


def load_archive_class(sdk):
    """Load the Archive class out of the SDK's archiver.rpy.

    The class sits in an `init python in archiver:` block of Ren'Py script,
    which plain Python cannot import; its lines are cut out, dedented and run
    with the few names that block would otherwise get from Ren'Py.

    Args:
        sdk: Root directory of a Ren'Py 6.18 SDK.

    Returns:
        The Archive class, as the launcher of that SDK uses it.
    """
    from cPickle import dumps, HIGHEST_PROTOCOL

    with open(os.path.join(sdk, "launcher", "game", "archiver.rpy"), "rb") as source:
        lines = source.read().lstrip(b"\xef\xbb\xbf").splitlines()

    start = next(i for i, line in enumerate(lines) if line.strip().startswith(b"class Archive("))
    indent = len(lines[start]) - len(lines[start].lstrip())
    block = [lines[start]]
    for line in lines[start + 1:]:
        if line.strip() and len(line) - len(line.lstrip()) <= indent:
            break
        block.append(line)

    namespace = {
        "dumps": dumps,
        "HIGHEST_PROTOCOL": HIGHEST_PROTOCOL,
        "_dict": dict,
        "_list": list,
    }
    exec(textwrap.dedent(b"\n".join(block)), namespace)
    return namespace["Archive"]


def main(argv):
    """Write the fixture with the archiver of the SDK named on the command line.

    Args:
        argv: Command line arguments, without the program name.
    """
    if sys.version_info[0] != 2:
        raise SystemExit("run this under the Python 2 of a Ren'Py 6.18 SDK:\n" + __doc__)
    if len(argv) != 1:
        raise SystemExit(__doc__)

    archive_class = load_archive_class(argv[0])
    work = tempfile.mkdtemp()
    try:
        archive = archive_class(FIXTURE)
        for number, (name, content) in enumerate(MEMBERS):
            member = os.path.join(work, str(number))
            with open(member, "wb") as out:
                out.write(content)
            archive.add(name, member)
        archive.close()
    finally:
        shutil.rmtree(work)
    print("wrote " + os.path.normpath(FIXTURE))


if __name__ == "__main__":
    main(sys.argv[1:])
