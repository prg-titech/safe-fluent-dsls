package prg.titech.chain;

import prg.titech.chain.token.Token;
import prg.titech.chain.visit.Visitable;

import java.util.List;

public interface Leaf extends Visitable {

    List<Token> getSourceTokens();

}
