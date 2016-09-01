#!/usr/bin/env python3
# ISC License (ISC)
#
# Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
# REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
# AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
# INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
# LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
# OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
# PERFORMANCE OF THIS SOFTWARE.

from plotly import graph_objs, plotly
import sys

try:
    keys = sys.argv[1].split(',')
except IndexError:
    sys.exit('Need at least one key')

try:
    values = sys.argv[2].split(',')
except IndexError:
    sys.exit('Need at least one value')

# Ensure that the number of keys (names) is equal to the number of values. i.e.,
# ensure that this is what happens:
#
# - a: 5
# - b: 11
# - c: 13
#
# and not something like:
#
# - a: 5
# - b: 12
# - c:
# - d:
if len(keys) != len(values):
    sys.exit('There are an unequal number of keys ({}) and values ({}'
        .format(len(keys), len(values)))

plotly.iplot({
    'data': [{
        'labels': keys,
        'values': values,
        'type': 'pie',
    }],
    'layout': {'title': 'Message Counts'},
})
