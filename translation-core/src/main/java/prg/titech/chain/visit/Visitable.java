package prg.titech.chain.visit;

public interface Visitable {
    <S> void accept(VoidVisitor<S> visitor, S state);

    <R, S> R accept(GenericVisitor<R, S> visitor, S state);
}
