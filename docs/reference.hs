import Control.Concurrent
import Data.Word
import Debug.Trace

-- Ratio of snapshots on an upper layer per snapshot on a lower layer
-- The numerical "base"
ratio = 4

kZero = 0

kMax = ratio - 1

data Layers a
  = End a
  | Node Int a (Layers a)

-- Get last layer value
getLast :: Layers a -> a
getLast (End x) = x
getLast (Node k x xs) = x

-- Insert next value
insert :: Show a => (a -> a -> a) -> a -> Layers a -> Layers a
insert merge nx old@(End _) =
  Node kZero nx old
insert merge nx (Node k ox tail)
  | k == kMax =
      -- Add layer for new value, merge old value below
      Node kZero nx (insert merge ox tail)
  | otherwise =
      -- Merge to current layer
      Node (k + 1) (merge nx ox) tail



-- Usage based on a simple counter
-- ===============================

-- Initial state
tip = End (0 :: Word)

-- Computes next state
inc = (+ 1)

-- Merges two layers
merge up down = up

-- Main loop
main = loop tip

-- that prints the overlay state at each tick
loop layers = do
  -- Show layers
  putStrLn $ showLayers layers
  -- Waits half secong
  threadDelay 500000
  -- Insert next value and loop
  loop $ (doInsert merge inc layers)

-- Computes next value from the current (last) value and inserts in the overlay
doInsert :: Show a => (a -> a -> a) -> (a -> a) -> Layers a -> Layers a
doInsert merge next layers =
    insert merge_ nx layers
  where
    -- Next value
    nx = next (getLast layers)
    merge_ x0 x1 =
      -- Uncomment `debug` to show when and which layers are beeing merged
      merge x0 x1 -- `debug` ("  (" ++ show x0 ++ "<--" ++ show x1 ++ ")")


-- Util
-- ====

debug = flip trace

toList :: Layers a -> [a]
toList (End x) = [x]
toList (Node k x xs) = x : toList xs

showLayers :: Show a => Layers a -> String
showLayers (End x) = "* " ++ show x ++ "  "
showLayers (Node k x xs) =
  showLayers xs
    -- ++ "["
    -- ++ show k
    -- ++ "]"
    ++ show x
    ++ "  "
