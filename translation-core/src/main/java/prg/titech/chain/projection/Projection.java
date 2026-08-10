package prg.titech.chain.projection;

import prg.titech.chain.token.Token;

public interface Projection {

    /**
     * The source code token which is being projected onto.
     * @return The source token belonging to the host language source code.
     */
    Token getSourceToken();

    /**
     * The DSL token which is being projected.
     * @return The target token belonging to the DSL translation.
     */
    Token getTargetToken();

    /**
     * The message which may or may not be projected onto the source token.
     * @return The projection message.
     */
    String getMessage();
}
