package main
import (
	"encoding/json"
	"fmt"
)
func main() {
	var prefs map[string]interface{}
	reqBody, _ := json.Marshal(map[string]interface{}{
		"prefs": prefs,
	})
	fmt.Println(string(reqBody))
}
