#include "alpha_export.h"

#include "../betawise-sdk/bw.h"

uint32_t alpha_export_append_session(const WodEditor_t* editor, uint32_t slot) {
    return bw_alphaword_append_text_block(slot, "WriteOrDie session", editor->bytes, editor->len);
}
