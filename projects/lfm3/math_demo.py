def is_prime(n):
    """Return True if n is a prime number, False otherwise."""
    if n <= 1:
        return False
    if n <= 3:
        return True
    if n % 2 == 0 or n % 3 == 0:
        return False
    i = 5
    while i * i <= n:
        if n % i == 0 or n % (i + 2) == 0:
            return False
        i += 6
    return True


if __name__ == "__main__":
    # Inspect the file itself: print its path and size
    import os
    print(f"File: {os.path.abspath(__file__)}")
    print(f"Size: {os.path.getsize(os.path.abspath(__file__))} bytes")

    # Example usage
    test_numbers = [2, 3, 4, 17, 18, 29, 100]
    for num in test_numbers:
        print(f"{num} is prime? {is_prime(num)}")
