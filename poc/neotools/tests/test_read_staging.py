import unittest

from neotools.read_staging import ReadStageBuffer


class ReadStageBufferTests(unittest.TestCase):
    def test_read_reuses_unconsumed_bytes_before_refilling(self) -> None:
        iterator = iter([b"ABCDEFGH"])
        buffer = ReadStageBuffer()

        first = buffer.read(3, lambda: next(iterator))
        second = buffer.read(4, lambda: next(iterator))

        self.assertEqual(first, b"ABC")
        self.assertEqual(second, b"DEFG")

    def test_read_can_span_multiple_refill_chunks(self) -> None:
        iterator = iter([b"ABCDEFGH", b"IJ"])
        buffer = ReadStageBuffer()

        result = buffer.read(10, lambda: next(iterator))

        self.assertEqual(result, b"ABCDEFGHIJ")

    def test_read_rejects_refill_chunks_larger_than_eight_bytes(self) -> None:
        buffer = ReadStageBuffer()

        with self.assertRaises(ValueError):
            buffer.read(1, lambda: b"012345678")


if __name__ == "__main__":
    unittest.main()
