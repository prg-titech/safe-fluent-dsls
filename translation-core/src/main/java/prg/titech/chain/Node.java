package prg.titech.chain;

import prg.titech.chain.token.TokenRange;
import prg.titech.chain.visit.Visitable;

import java.util.Optional;

public interface Node extends Visitable {

    Optional<TokenRange> getTokenRange();

}
