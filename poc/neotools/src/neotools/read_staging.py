from collections.abc import Callable


class ReadStageBuffer:
    def __init__(self) -> None:
        self._pending = bytearray()

    def read(self, requested: int, refill: Callable[[], bytes]) -> bytes:
        if requested < 0:
            raise ValueError("requested must be non-negative")

        while len(self._pending) < requested:
            chunk = refill()
            if not 1 <= len(chunk) <= 8:
                raise ValueError("refill chunk must contain between 1 and 8 bytes")
            self._pending.extend(chunk)

        result = bytes(self._pending[:requested])
        del self._pending[:requested]
        return result
