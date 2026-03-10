import mysql.connector

try:
    conn = mysql.connector.connect(host="127.0.0.1", user="root", port=3307)
    cursor = conn.cursor()
    cursor.execute("CREATE DATABASE beads;")
    print("Database beads created.")
except Exception as e:
    print(f"Error: {e}")
