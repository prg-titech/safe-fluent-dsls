package prg.titech.chain.visit;

import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;

public interface Visitable {
    void accept(ChainTraverser traverser);

    void accept(Frame context, ChainVisitor visitor);

    <S> void accept(VoidVisitor<S> visitor, S state);

    <R, S> R accept(GenericVisitor<R, S> visitor, S state);
}
