import unittest

from neotools.alphaword_transfer import ChunkExchange, reconstruct_file_from_exchanges


class AlphaWordTransferTests(unittest.TestCase):
    def test_reconstruct_file_from_start_and_chunk_exchanges(self) -> None:
        data = reconstruct_file_from_exchanges(
            start_response=bytes.fromhex("53 00 00 00 05 00 00 58"),
            chunks=[
                ChunkExchange(
                    response=bytes.fromhex("4d 00 00 00 03 00 c6 16"),
                    payload=b"ABC",
                ),
                ChunkExchange(
                    response=bytes.fromhex("4d 00 00 00 02 00 89 d8"),
                    payload=b"DE",
                ),
            ],
        )

        self.assertEqual(data, b"ABCDE")

    def test_reconstruct_file_rejects_bad_status_or_payload_mismatch(self) -> None:
        with self.assertRaises(ValueError):
            reconstruct_file_from_exchanges(
                start_response=bytes.fromhex("44 00 00 00 05 00 00 49"),
                chunks=[],
            )

        with self.assertRaises(ValueError):
            reconstruct_file_from_exchanges(
                start_response=bytes.fromhex("53 00 00 00 03 00 00 56"),
                chunks=[
                    ChunkExchange(
                        response=bytes.fromhex("4d 00 00 00 03 00 c6 16"),
                        payload=b"AB",
                    )
                ],
            )

    def test_reconstruct_file_rejects_chunk_checksum_mismatch(self) -> None:
        with self.assertRaises(ValueError):
            reconstruct_file_from_exchanges(
                start_response=bytes.fromhex("53 00 00 00 03 00 00 56"),
                chunks=[
                    ChunkExchange(
                        response=bytes.fromhex("4d 00 00 00 03 00 c7 17"),
                        payload=b"ABC",
                    )
                ],
            )


if __name__ == "__main__":
    unittest.main()
