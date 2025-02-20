import { StatusBar } from "expo-status-bar";
import { StyleSheet, Text, View, TextInput, Button } from "react-native";
import { calculateFee } from "./modules/my-rust-module";
import { useState } from "react";

export default function App() {
  const [nTxin, setNTxin] = useState("");
  const [nTxout, setNTxout] = useState("");
  const [nSpend, setNSpend] = useState("");
  const [nSout, setNSout] = useState("");

  const [value, setValue] = useState<null | number>(null);

  const handleCalculateFee = async () => {
    // Convert inputs to numbers before passing to the Rust function
    const fee = await calculateFee(
      parseInt(nTxin, 10),
      parseInt(nTxout, 10),
      parseInt(nSpend, 10),
      parseInt(nSout, 10),
    );
    setValue(fee);
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Transaction Fee Calculator</Text>

      {/* Input for n_txin */}
      <TextInput
        style={styles.input}
        placeholder="n_txin"
        value={nTxin}
        onChangeText={setNTxin}
        keyboardType="numeric"
      />

      {/* Input for n_txout */}
      <TextInput
        style={styles.input}
        placeholder="n_txout"
        value={nTxout}
        onChangeText={setNTxout}
        keyboardType="numeric"
      />

      {/* Input for n_spend */}
      <TextInput
        style={styles.input}
        placeholder="n_spend"
        value={nSpend}
        onChangeText={setNSpend}
        keyboardType="numeric"
      />

      {/* Input for n_sout */}
      <TextInput
        style={styles.input}
        placeholder="n_sout"
        value={nSout}
        onChangeText={setNSout}
        keyboardType="numeric"
      />

      <Button title="Calculate Fee" onPress={handleCalculateFee} />

      <Text style={styles.result}>
        {value === null ? "Result will appear here..." : `Fee: ${value}`}
      </Text>

      <StatusBar style="auto" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#FFFFFF",
    alignItems: "center",
    justifyContent: "center",
    padding: 20,
  },
  title: {
    fontSize: 28,
    marginBottom: 20,
  },
  input: {
    width: "80%",
    height: 40,
    padding: 8,
    borderWidth: 1,
    borderColor: "#CCC",
    marginBottom: 10,
    borderRadius: 5,
  },
  result: {
    marginTop: 20,
    fontSize: 24,
  },
});
