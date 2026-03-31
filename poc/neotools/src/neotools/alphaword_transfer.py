from dataclasses import dataclass

from neotools.updater_responses import parse_updater_response


@dataclass(frozen=True)
class ChunkExchange:
    response: bytes
    payload: bytes


def reconstruct_file_from_exchanges(*, start_response: bytes, chunks: list[ChunkExchange]) -> bytes:
    start = parse_updater_response(start_response)
    if start.status != 0x53:
        raise ValueError("retrieve start response must use status 0x53")

    expected_length = start.argument
    data = bytearray()

    for chunk in chunks:
        parsed = parse_updater_response(chunk.response)
        if parsed.status != 0x4D:
            raise ValueError("retrieve chunk response must use status 0x4d")
        if parsed.argument != len(chunk.payload):
            raise ValueError("retrieve chunk payload length does not match response header")
        if (sum(chunk.payload) & 0xFFFF) != parsed.trailing:
            raise ValueError("retrieve chunk payload checksum mismatch")

        data.extend(chunk.payload)
        if len(data) == expected_length:
            return bytes(data)

    if len(data) != expected_length:
        raise ValueError("retrieve exchanges ended before expected byte count was reached")

    return bytes(data)
